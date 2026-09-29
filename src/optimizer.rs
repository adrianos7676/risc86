use crate::{CPUSTATEREG, JUNKREG0};

enum FlagProducer {
    Direct(u8),
    Shift(u8),
}

fn reads_register(instruction: u32, register: u8) -> bool {
    let opcode = instruction & 0x7f;

    let rs1 = ((instruction >> 15) & 0x1f) as u8;
    let rs2 = ((instruction >> 20) & 0x1f) as u8;

    match opcode {
        // R-type
        0b0110011 => rs1 == register || rs2 == register,

        // I-type ALU
        0b0010011 => rs1 == register,

        // LOAD
        0b0000011 => rs1 == register,

        // STORE
        0b0100011 => rs1 == register || rs2 == register,

        // BRANCH
        0b1100011 => rs1 == register || rs2 == register,

        // JALR
        0b1100111 => rs1 == register,

        // JAL
        0b1101111 => false,

        // LUI
        0b0110111 => false,

        // AUIPC
        0b0010111 => false,

        // SYSTEM / ECALL
        //
        // Linux RISC-V syscall ABI:
        // a0-a5 = syscall arguments
        // a7    = syscall number
        0b1110011 => matches!(register, 10..=15 | 17),

        _ => false,
    }
}

fn writes_register(instruction: u32, register: u8) -> bool {
    let opcode = instruction & 0x7f;

    let rd = ((instruction >> 7) & 0x1f) as u8;

    match opcode {
        // R-type
        0b0110011 => rd == register,

        // I-type ALU
        0b0010011 => rd == register,

        // LOAD
        0b0000011 => rd == register,

        // JAL
        0b1101111 => rd == register,

        // JALR
        0b1100111 => rd == register,

        // LUI
        0b0110111 => rd == register,

        // AUIPC
        0b0010111 => rd == register,

        // STORE
        0b0100011 => false,

        // BRANCH
        0b1100011 => false,

        // SYSTEM
        0b1110011 => false,

        _ => false,
    }
}

fn read_flag_producer(instruction: u32) -> Option<FlagProducer> {
    let opcode = instruction & 0x7f;
    let funct3 = (instruction >> 12) & 0x7;
    let rd = ((instruction >> 7) & 0x1f) as u8;

    if rd != JUNKREG0 {
        return None;
    }

    match (opcode, funct3) {
        // SLTU -> CF
        (0b0110011, 0b011) => Some(FlagProducer::Direct(0)),

        // SLLI -> shifted flag
        (0b0010011, 0b001) => {
            let shift = ((instruction >> 20) & 0x3f) as u8;

            Some(FlagProducer::Shift(shift))
        }

        _ => None,
    }
}

fn read_cpu_state_mask(instruction: u32) -> Option<u64> {
    let opcode = instruction & 0x7f;
    let funct3 = (instruction >> 12) & 0x7;
    let rs1 = ((instruction >> 15) & 0x1f) as u8;

    // ANDI
    if opcode != 0b0010011 || funct3 != 0b111 {
        return None;
    }

    if rs1 != CPUSTATEREG {
        return None;
    }

    let imm = ((instruction as i32) >> 20) as i64;

    Some(imm as u64)
}

fn flag_from_shift(shift: Option<u8>) -> &'static str {
    match shift {
        None => "CF",
        Some(6) => "ZF",
        Some(7) => "SF",
        Some(4) => "AF",
        Some(11) => "OF",
        Some(2) => "PF",
        Some(_) => "UNKNOWN",
    }
}

pub fn is_cpu_state_write(instruction: u32) -> bool {
    let opcode = instruction & 0x7f;
    let funct3 = (instruction >> 12) & 0x7;
    let rd = (instruction >> 7) & 0x1f;
    let rs1 = (instruction >> 15) & 0x1f;
    let rs2 = (instruction >> 20) & 0x1f;

    opcode == 0b0110011
        && funct3 == 0b110
        && rd == CPUSTATEREG as u32
        && (rs1 == CPUSTATEREG as u32 || rs2 == CPUSTATEREG as u32)
}

pub fn read_address_from_where_cpu_flag_is_set(instruction: u32) -> u8 {
    let rs1 = ((instruction >> 15) & 0x1f) as u8;
    let rs2 = ((instruction >> 20) & 0x1f) as u8;

    if rs1 == CPUSTATEREG { rs2 } else { rs1 }
}

fn writes_to_register(instruction: u32, register: u8) -> bool {
    let opcode = instruction & 0x7f;

    match opcode {
        // R/I/U/J type — rd
        0b0110011 | 0b0010011 | 0b0000011 | 0b0110111 | 0b0010111 | 0b1101111 | 0b1100111 => {
            let rd = ((instruction >> 7) & 0x1f) as u8;

            rd == register
        }

        _ => false,
    }
}

fn remove_unused_writes_to_cpustatereg(
    index: usize,
    instruction: u32,
    code: &[u32],
    cpu_state_writes: &mut [Option<usize>; 64],
    cpu_state_dead_candidates: &mut Vec<usize>,
) {
    // ============================================================
    // CPUSTATE READ
    // ============================================================

    if let Some(mask) = read_cpu_state_mask(instruction) {
        for flag in 0..64 {
            if mask & (1u64 << flag) != 0 {
                cpu_state_writes[flag] = None;
            }
        }
    }

    // ============================================================
    // CPUSTATE WRITE
    // ============================================================

    if is_cpu_state_write(instruction) {
        let register = read_address_from_where_cpu_flag_is_set(instruction);

        for i in (0..index).rev() {
            let previous_instruction = code[i];

            if writes_to_register(previous_instruction, register) {
                if let Some(producer) = read_flag_producer(previous_instruction) {
                    let flag = match producer {
                        FlagProducer::Direct(flag) => flag as usize,

                        FlagProducer::Shift(shift) => shift as usize,
                    };

                    if let Some(previous_write) = cpu_state_writes[flag] {
                        // ------------------------------------------------
                        // B-type conditional branch is a control-flow
                        // boundary for this simple linear analysis.
                        //
                        // JAL/JALR do NOT block this optimization.
                        // ------------------------------------------------

                        let has_conditional_branch = (previous_write + 1..index).any(|i| {
                            let opcode = code[i] & 0x7f;

                            opcode == 0b1100011
                        });

                        if !has_conditional_branch {
                            println!(
                                "candidate dead CPUSTATE write at {} -> {}",
                                previous_write,
                                flag_from_shift(Some(flag as u8))
                            );

                            // Do NOT delete it yet.
                            //
                            // PASS 3 may discover that this instruction
                            // is actually required by a later CPUSTATE read.
                            cpu_state_dead_candidates.push(previous_write);
                        }
                    }

                    cpu_state_writes[flag] = Some(index);
                }

                break;
            }
        }
    }
}

fn run_optimize(code: Vec<u32>) -> Vec<u32> {
    let mut live_instructions = vec![false; code.len()];

    let mut cpu_state_writes = [None::<usize>; 64];

    // CPUSTATE instructions which MAY be dead.
    //
    // We cannot delete them yet because PASS 3 may discover that
    // a later branch actually depends on them.
    let mut cpu_state_dead_candidates = Vec::new();

    let mut dead_instructions = Vec::new();

    // ============================================================
    // PASS 1
    //
    // Find CPUSTATE writes which are candidates for removal.
    // Do NOT remove anything yet.
    // ============================================================

    for (index, instruction) in code.iter().enumerate() {
        remove_unused_writes_to_cpustatereg(
            index,
            *instruction,
            &code,
            &mut cpu_state_writes,
            &mut cpu_state_dead_candidates,
        );
    }

    // ============================================================
    // PASS 2
    //
    // Mark instructions with observable side effects as LIVE.
    // ============================================================

    for (index, instruction) in code.iter().enumerate() {
        let opcode = instruction & 0x7f;

        match opcode {
            // STORE
            0b0100011 => {
                live_instructions[index] = true;
            }

            // CONDITIONAL BRANCH
            0b1100011 => {
                live_instructions[index] = true;
            }

            // JAL
            0b1101111 => {
                live_instructions[index] = true;
            }

            // JALR
            0b1100111 => {
                live_instructions[index] = true;
            }

            // SYSTEM / ECALL
            0b1110011 => {
                live_instructions[index] = true;
            }

            _ => {}
        }
    }

    // ============================================================
    // PASS 3
    //
    // Propagate dependencies backwards.
    // ============================================================

    loop {
        let mut changed = false;

        for index in (0..code.len()).rev() {
            if !live_instructions[index] {
                continue;
            }

            let instruction = code[index];

            // ============================================================
            // CPUSTATE DEPENDENCY
            // ============================================================

            if let Some(mask) = read_cpu_state_mask(instruction) {
                for flag in 0..64 {
                    if mask & (1u64 << flag) == 0 {
                        continue;
                    }

                    // Search backwards through CPUSTATE writes.
                    //
                    // IMPORTANT:
                    // The nearest CPUSTATE write does NOT necessarily contain
                    // the flag we are looking for.
                    //
                    // Example:
                    //
                    //     or s8, s8, s9    // PF -> bit 2
                    //     ...
                    //     andi s9, s8, 0x40 // ZF -> bit 6
                    //
                    // The PF write is closer, but it is not the producer
                    // of bit 6. Therefore we MUST continue searching.

                    for state_write_index in (0..index).rev() {
                        if !is_cpu_state_write(code[state_write_index]) {
                            continue;
                        }

                        let source_register =
                            read_address_from_where_cpu_flag_is_set(code[state_write_index]);

                        // Find the nearest instruction which writes the source
                        // register used by this CPUSTATE write.
                        let Some(producer_index) = (0..state_write_index)
                            .rev()
                            .find(|&i| writes_to_register(code[i], source_register))
                        else {
                            continue;
                        };

                        let Some(producer) = read_flag_producer(code[producer_index]) else {
                            continue;
                        };

                        let produced_flag = match producer {
                            FlagProducer::Direct(flag) => flag as usize,
                            FlagProducer::Shift(shift) => shift as usize,
                        };

                        // This CPUSTATE write does not contain the flag we need.
                        //
                        // DO NOT break here.
                        //
                        // There may be an older CPUSTATE write containing the
                        // requested flag.
                        if produced_flag != flag {
                            continue;
                        }

                        // ----------------------------------------------------
                        // We found the actual CPUSTATE write which contributes
                        // the requested flag.
                        // ----------------------------------------------------

                        if !live_instructions[state_write_index] {
                            live_instructions[state_write_index] = true;
                            changed = true;
                        }

                        // The instruction which generated the flag is required.
                        if !live_instructions[producer_index] {
                            live_instructions[producer_index] = true;
                            changed = true;
                        }

                        // We found the correct producer for this flag.
                        break;
                    }
                }
            }

            // ========================================================
            // NORMAL REGISTER DEPENDENCY
            // ========================================================

            for register in 0..32 {
                let register = register as u8;

                // CPUSTATE has its own dependency analysis above.
                if register == CPUSTATEREG as u8 {
                    continue;
                }

                if !reads_register(instruction, register) {
                    continue;
                }

                // Find the nearest producer before this instruction.
                let producer = (0..index)
                    .rev()
                    .find(|&producer_index| writes_register(code[producer_index], register));

                if let Some(producer_index) = producer {
                    if !live_instructions[producer_index] {
                        live_instructions[producer_index] = true;

                        changed = true;
                    }
                }
            }
        }

        if !changed {
            break;
        }
    }

    // ============================================================
    // PASS 4
    //
    // Now, and only now, actually remove dead CPUSTATE candidates.
    //
    // If PASS 3 marked one LIVE, it survives.
    // ============================================================

    for index in cpu_state_dead_candidates {
        if !live_instructions[index] {
            dead_instructions.push(index);
        }
    }

    // ============================================================
    // PASS 5
    //
    // Everything else which is not LIVE is dead.
    // ============================================================

    for index in 0..code.len() {
        if !live_instructions[index] {
            dead_instructions.push(index);
        }
    }

    dead_instructions.sort_unstable();
    dead_instructions.dedup();

    println!("dead instructions: {:?}", dead_instructions);

    code.into_iter()
        .enumerate()
        .filter_map(|(index, instruction)| {
            if dead_instructions.binary_search(&index).is_ok() {
                None
            } else {
                Some(instruction)
            }
        })
        .collect()
}

pub fn optimize(mut code: Vec<u32>) -> Vec<u32> {
    loop {
        let optimised = run_optimize(code.clone());

        if optimised == code {
            break;
        }

        code = optimised;
    }

    code
}
