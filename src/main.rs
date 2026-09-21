use std::{env, fs::File, io::Write, path::PathBuf, process::Command};

mod elf;
mod encode;

#[derive(Debug, Clone, Copy)]
enum X86Reg {
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rsi,
    Rdi,
    Rbp,
    Rsp,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

#[derive(Debug, Clone, Copy)]
enum RiscVReg {
    X8,
    X9,
    X10,
    X11,
    X12,
    X13,
    X14,
    X15,
    X16,
    X17,
    X18,
    X19,
    X20,
    X21,
    X22,
    X23,
}

impl X86operation {
    fn modrm_len(code: &[u8], offset: usize) -> usize {
        let modrm = code[offset];
        let mode = modrm >> 6;
        let rm = modrm & 0b111;

        let mut len = 1;

        if mode != 3 && rm == 4 {
            len += 1;

            let sib = code[offset + 1];
            let base = sib & 0b111;

            if mode == 0 && base == 5 {
                len += 4;
            }
        }

        match mode {
            0 => {
                if rm == 5 {
                    len += 4;
                }
            }
            1 => len += 1,
            2 => len += 4,
            _ => {}
        }

        len
    }
    pub fn decode(code: &[u8], offset: usize) -> DecodedInstruction {
        let mut pos = offset;
        let mut rex = false;

        dbg!(
            offset,
            code[offset],
            &code[offset.saturating_sub(10)..(offset + 10).min(code.len())]
        );

        if (code[pos] & 0xF0) == 0x40 {
            rex = true;
            pos += 1;
        }

        match code[pos] {
            0x0F => match code.get(pos + 1..) {
                Some([0x05, ..]) => DecodedInstruction {
                    operation: Self::Syscall,
                    len: (pos - offset) + 2,
                },
                _ => todo!(),
            },

            0xF3 => match code.get(pos + 1..) {
                Some([0x0F, 0x1E, 0xFA, ..]) => DecodedInstruction {
                    operation: Self::Endbr64,
                    len: (pos - offset) + 4,
                },
                _ => todo!(),
            },

            0x88..=0x8B => DecodedInstruction {
                operation: Self::Mov,
                len: (pos - offset) + 1 + Self::modrm_len(code, pos + 1),
            },

            0xC6..=0xC7 => DecodedInstruction {
                operation: Self::Mov,
                len: todo!(),
            },

            0xB8..=0xBF => DecodedInstruction {
                operation: Self::Mov,
                len: if rex { 10 } else { 5 },
            },

            0x8D => DecodedInstruction {
                operation: Self::Lea,
                len: (pos - offset) + 1 + Self::modrm_len(code, pos + 1),
            },

            0x00..=0x05 => DecodedInstruction {
                operation: Self::Add,
                len: match code[pos] {
                    0x00..=0x03 => (pos - offset) + 1 + Self::modrm_len(code, pos + 1),
                    0x04 => (pos - offset) + 2,
                    0x05 => (pos - offset) + 5,
                    _ => unreachable!(),
                },
            },

            0x28..=0x2D => DecodedInstruction {
                operation: Self::Sub,
                len: match code[pos] {
                    0x28..=0x2B => (pos - offset) + 1 + Self::modrm_len(code, pos + 1),
                    0x2C => (pos - offset) + 2,
                    0x2D => (pos - offset) + 5,
                    _ => unreachable!(),
                },
            },

            0x30..=0x33 => DecodedInstruction {
                operation: Self::Xor,
                len: (pos - offset) + 1 + Self::modrm_len(code, pos + 1),
            },

            0x34 => DecodedInstruction {
                operation: Self::Xor,
                len: (pos - offset) + 2,
            },

            0x35 => DecodedInstruction {
                operation: Self::Xor,
                len: (pos - offset) + 5,
            },

            0x20..=0x25 => DecodedInstruction {
                operation: Self::And,
                len: match code[pos] {
                    0x20..=0x23 => (pos - offset) + 1 + Self::modrm_len(code, pos + 1),
                    0x24 => (pos - offset) + 2,
                    0x25 => (pos - offset) + 5,
                    _ => unreachable!(),
                },
            },

            0x08..=0x0D => DecodedInstruction {
                operation: Self::Or,
                len: match code[pos] {
                    0x08..=0x0B => (pos - offset) + 1 + Self::modrm_len(code, pos + 1),
                    0x0C => (pos - offset) + 2,
                    0x0D => (pos - offset) + 5,
                    _ => unreachable!(),
                },
            },

            0x38..=0x3D => DecodedInstruction {
                operation: Self::Cmp,
                len: match code[pos] {
                    0x38..=0x3B => (pos - offset) + 1 + Self::modrm_len(code, pos + 1),
                    0x3C => (pos - offset) + 2,
                    0x3D => (pos - offset) + 5,
                    _ => unreachable!(),
                },
            },

            0x83 => DecodedInstruction {
                operation: Self::And,
                len: (pos - offset) + 1 + Self::modrm_len(code, pos + 1) + 1,
            },
            0x84..=0x85 => DecodedInstruction {
                operation: Self::Test,
                len: todo!(),
            },

            0x50..=0x57 => DecodedInstruction {
                operation: Self::Push,
                len: (pos - offset) + 1,
            },

            0x58..=0x5F => DecodedInstruction {
                operation: Self::Pop,
                len: (pos - offset) + 1,
            },

            0xE8 => DecodedInstruction {
                operation: Self::Call,
                len: (pos - offset) + 5,
            },

            0xC3 => DecodedInstruction {
                operation: Self::Ret,
                len: (pos - offset) + 1,
            },

            0xC2 => DecodedInstruction {
                operation: Self::Ret,
                len: (pos - offset) + 3,
            },

            0xE9 => DecodedInstruction {
                operation: Self::Jmp,
                len: (pos - offset) + 5,
            },

            0xEB => DecodedInstruction {
                operation: Self::Jmp,
                len: (pos - offset) + 2,
            },

            0x70..=0x7F => DecodedInstruction {
                operation: Self::ConditionalJump,
                len: (pos - offset) + 2,
            },

            0x90 => DecodedInstruction {
                operation: Self::Nop,
                len: (pos - offset) + 1,
            },

            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X86operation {
    Mov,
    Lea,
    Add,
    Sub,
    Xor,
    And,
    Or,
    Cmp,
    Test,
    Push,
    Pop,
    Call,
    Ret,
    Jmp,
    ConditionalJump,
    Syscall,
    Nop,
    Endbr64,
}

#[derive(Debug)]
pub struct DecodedInstruction {
    operation: X86operation,
    len: usize,
}

impl X86Reg {
    fn from_modrm(byte: u8, rex_b: bool) -> Self {
        let mut reg = byte & 0b111;

        if rex_b {
            reg += 8;
        }

        match reg {
            0 => Self::Rax,
            1 => Self::Rcx,
            2 => Self::Rdx,
            3 => Self::Rbx,
            4 => Self::Rsp,
            5 => Self::Rbp,
            6 => Self::Rsi,
            7 => Self::Rdi,
            8 => Self::R8,
            9 => Self::R9,
            10 => Self::R10,
            11 => Self::R11,
            12 => Self::R12,
            13 => Self::R13,
            14 => Self::R14,
            15 => Self::R15,
            _ => unreachable!(),
        }
    }

    fn from_modrm_reg(byte: u8, rex_r: bool) -> Self {
        let mut reg = (byte >> 3) & 0b111;

        if rex_r {
            reg += 8;
        }

        match reg {
            0 => Self::Rax,
            1 => Self::Rcx,
            2 => Self::Rdx,
            3 => Self::Rbx,
            4 => Self::Rsp,
            5 => Self::Rbp,
            6 => Self::Rsi,
            7 => Self::Rdi,
            8 => Self::R8,
            9 => Self::R9,
            10 => Self::R10,
            11 => Self::R11,
            12 => Self::R12,
            13 => Self::R13,
            14 => Self::R14,
            15 => Self::R15,
            _ => unreachable!(),
        }
    }

    fn to_index(self) -> usize {
        match self {
            Self::Rax => 0,
            Self::Rbx => 1,
            Self::Rcx => 2,
            Self::Rdx => 3,
            Self::Rsi => 4,
            Self::Rdi => 5,
            Self::Rbp => 6,
            Self::Rsp => 7,
            Self::R8 => 8,
            Self::R9 => 9,
            Self::R10 => 10,
            Self::R11 => 11,
            Self::R12 => 12,
            Self::R13 => 13,
            Self::R14 => 14,
            Self::R15 => 15,
        }
    }

    fn to_riscv(self) -> u8 {
        match self {
            Self::Rax => 8,
            Self::Rbx => 9,
            Self::Rcx => 10,
            Self::Rdx => 11,
            Self::Rsi => 12,
            Self::Rdi => 13,
            Self::Rbp => 14,
            Self::Rsp => 15,
            Self::R8 => 16,
            Self::R9 => 17,
            Self::R10 => 18,
            Self::R11 => 19,
            Self::R12 => 20,
            Self::R13 => 21,
            Self::R14 => 22,
            Self::R15 => 23,
        }
    }
}

impl SysCall {
    fn from_x86(code: u64) -> Self {
        match code {
            1 => Self::Write,
            60 => Self::Exit,
            _ => todo!(),
        }
    }

    fn to_riscv(&self) -> u64 {
        match self {
            Self::Write => 64,
            Self::Exit => 93,
        }
    }
}

enum SysCall {
    Write,
    Exit,
}

fn main() {
    fn load_u64(riscv_register: u8, value: u64, riscv_code: &mut Vec<u32>) {
        let mut chunks = [0i64; 6];
        let mut value = value as i128;

        for i in (0..6).rev() {
            let mut chunk = (value & 0xfff) as i64;

            if chunk >= 0x800 {
                chunk -= 0x1000;
            }

            chunks[i] = chunk;
            value = (value - chunk as i128) >> 12;
        }

        riscv_code.push(encode::encode_addi(riscv_register, 0, chunks[0] as i32));

        for chunk in &chunks[1..] {
            riscv_code.push(encode::encode_slli(riscv_register, riscv_register, 12));

            if *chunk != 0 {
                riscv_code.push(encode::encode_addi(
                    riscv_register,
                    riscv_register,
                    *chunk as i32,
                ));
            }
        }
    }

    let mut riscv_code: Vec<u32> = Vec::new();
    let mut registers = [0u64; 16];
    let mut lea_fixups: Vec<(usize, u8, u64)> = Vec::new();

    let args: Vec<String> = env::args().collect();
    dbg!(&args);

    if args.len() >= 2 {
        let input_file = PathBuf::from(&args[1]);

        dbg!(&input_file);

        if let Ok(bytes) = std::fs::read(&input_file) {
            dbg!(&bytes[0..4]);

            if &bytes[0..4] != b"\x7fELF" {
                panic!("Not an ELF file");
            }

            if &bytes[4] != &2 {
                panic!("Nont an ELF64 file");
            }

            if &bytes[5] != &1 {
                panic!("Not little-endian");
            }

            let header = elf::Elf64Header::from_bytes(&bytes[0..64]);

            dbg!(&header);

            let mut segments: Vec<elf::Elf64ProgramHeader> = Vec::new();

            for i in 0..header.e_phnum {
                let offset = header.e_phoff as usize + i as usize * header.e_phentsize as usize;

                let program_header = elf::Elf64ProgramHeader::from_bytes(
                    &bytes[offset..offset + header.e_phentsize as usize],
                );

                dbg!(&program_header);

                if program_header.p_type == 1 {
                    segments.push(program_header);
                }
            }

            let mut riscv_data: Vec<u8> = Vec::new();

            for program_header in segments.iter() {
                if program_header.p_vaddr <= header.e_entry
                    && header.e_entry < program_header.p_vaddr + program_header.p_memsz
                {
                    let file_offset =
                        program_header.p_offset + (header.e_entry - program_header.p_vaddr);

                    let code_start = file_offset as usize;
                    let code_end = (program_header.p_offset + program_header.p_filesz) as usize;

                    let code = &bytes[code_start..code_end];

                    dbg!(code);

                    let mut code_offset = 0;
                    while code_offset < code.len() {
                        dbg!(code_offset);
                        dbg!(code[code_offset]);

                        //read the operation
                        let operation = X86operation::decode(code, code_offset);
                        dbg!(operation);

                        let operation = X86operation::decode(code, code_offset);
                        dbg!(&operation);

                        match operation.operation {
                            X86operation::Syscall => {
                                let syscall_number = registers[0];

                                let syscall = SysCall::from_x86(syscall_number);
                                let riscv_syscall = syscall.to_riscv();

                                match syscall {
                                    SysCall::Write => {
                                        riscv_code.push(encode::encode_addi(5, 13, 0));
                                        riscv_code.push(encode::encode_addi(6, 12, 0));
                                        riscv_code.push(encode::encode_addi(12, 11, 0));

                                        riscv_code.push(encode::encode_addi(11, 6, 0));
                                        riscv_code.push(encode::encode_addi(10, 5, 0));
                                    }

                                    SysCall::Exit => {
                                        riscv_code.push(encode::encode_addi(10, 13, 0));
                                    }
                                }

                                riscv_code.push(encode::encode_addi(17, 0, riscv_syscall as i32));

                                riscv_code.push(encode::encode_ecall());

                                code_offset += operation.len;
                            }

                            X86operation::Mov => match code[code_offset] {
                                0xB8..=0xBF => {
                                    let rex_w = (code[code_offset] & 0x08) != 0;
                                    let rex_b = (code[code_offset] & 0x01) != 0;

                                    let opcode_offset = if (code[code_offset] & 0xF0) == 0x40 {
                                        code_offset + 1
                                    } else {
                                        code_offset
                                    };

                                    let opcode = code[opcode_offset];

                                    let mut register = opcode & 0b111;

                                    if rex_b {
                                        register += 8;
                                    }

                                    let x86_register = X86Reg::from_modrm(register, false);
                                    let riscv_register = x86_register.to_riscv();

                                    let value = if rex_w {
                                        u64::from_le_bytes(
                                            code[opcode_offset + 1..opcode_offset + 9]
                                                .try_into()
                                                .unwrap(),
                                        )
                                    } else {
                                        u32::from_le_bytes(
                                            code[opcode_offset + 1..opcode_offset + 5]
                                                .try_into()
                                                .unwrap(),
                                        ) as u64
                                    };

                                    registers[x86_register.to_index()] = value;

                                    if rex_w {
                                        load_u64(riscv_register, value, &mut riscv_code);
                                    } else {
                                        riscv_code.push(encode::encode_addi(
                                            riscv_register,
                                            0,
                                            value as i32,
                                        ));
                                    }

                                    code_offset += operation.len;
                                }

                                0x48 => match code[code_offset + 1] {
                                    0xB8..=0xBF => {
                                        let opcode = code[code_offset + 1];

                                        let x86_register = match opcode & 0b111 {
                                            0 => X86Reg::Rax,
                                            1 => X86Reg::Rcx,
                                            2 => X86Reg::Rdx,
                                            3 => X86Reg::Rbx,
                                            4 => X86Reg::Rsp,
                                            5 => X86Reg::Rbp,
                                            6 => X86Reg::Rsi,
                                            7 => X86Reg::Rdi,
                                            _ => unreachable!(),
                                        };

                                        let value = u64::from_le_bytes(
                                            code[code_offset + 2..code_offset + 10]
                                                .try_into()
                                                .unwrap(),
                                        );

                                        let riscv_register = x86_register.to_riscv();

                                        registers[x86_register.to_index()] = value;

                                        load_u64(riscv_register, value, &mut riscv_code);

                                        code_offset += operation.len;
                                    }

                                    0x89 => {
                                        let modrm = code[code_offset + 2];

                                        let destination = X86Reg::from_modrm(modrm, false);
                                        let source = X86Reg::from_modrm_reg(modrm, false);

                                        let riscv_destination = destination.to_riscv();
                                        let riscv_source = source.to_riscv();

                                        registers[destination.to_index()] =
                                            registers[source.to_index()];

                                        riscv_code.push(encode::encode_addi(
                                            riscv_destination,
                                            riscv_source,
                                            0,
                                        ));

                                        code_offset += operation.len;
                                    }
                                    0xC7 => {
                                        let modrm = code[code_offset + 2];

                                        let x86_register = X86Reg::from_modrm(modrm, false);
                                        let riscv_register = x86_register.to_riscv();

                                        let value = u32::from_le_bytes(
                                            code[code_offset + 3..code_offset + 7]
                                                .try_into()
                                                .unwrap(),
                                        )
                                            as u64;

                                        registers[x86_register.to_index()] = value;

                                        dbg!(riscv_register);
                                        dbg!(value);
                                        dbg!(registers);

                                        riscv_code.push(encode::encode_addi(
                                            riscv_register,
                                            0,
                                            value as i32,
                                        ));

                                        code_offset += operation.len;
                                    }

                                    _ => todo!(),
                                },

                                0x49 => match code[code_offset + 1] {
                                    0x89 => {
                                        let modrm = code[code_offset + 2];

                                        let destination = X86Reg::from_modrm(modrm, true);
                                        let source = X86Reg::from_modrm_reg(modrm, false);

                                        let riscv_destination = destination.to_riscv();
                                        let riscv_source = source.to_riscv();

                                        registers[destination.to_index()] =
                                            registers[source.to_index()];

                                        riscv_code.push(encode::encode_addi(
                                            riscv_destination,
                                            riscv_source,
                                            0,
                                        ));

                                        code_offset += operation.len;
                                    }

                                    _ => todo!(),
                                },

                                0x4C => match code[code_offset + 1] {
                                    0x89 => {
                                        let modrm = code[code_offset + 2];

                                        let destination = X86Reg::from_modrm(modrm, false);
                                        let source = X86Reg::from_modrm_reg(modrm, true);

                                        let riscv_destination = destination.to_riscv();
                                        let riscv_source = source.to_riscv();

                                        registers[destination.to_index()] =
                                            registers[source.to_index()];

                                        riscv_code.push(encode::encode_addi(
                                            riscv_destination,
                                            riscv_source,
                                            0,
                                        ));

                                        code_offset += operation.len;
                                    }

                                    _ => todo!(),
                                },
                                _ => todo!(),
                            },

                            X86operation::Lea => {
                                if code[code_offset] != 0x48 || code[code_offset + 1] != 0x8D {
                                    todo!();
                                }

                                let modrm = code[code_offset + 2];
                                let x86_register = X86Reg::from_modrm_reg(modrm, false);

                                let mode = modrm >> 6;
                                let rm = modrm & 0b111;

                                if mode != 0b00 || rm != 0b101 {
                                    todo!();
                                }

                                let disp = i32::from_le_bytes(
                                    code[code_offset + 3..code_offset + 7].try_into().unwrap(),
                                );

                                let next_rip =
                                    header.e_entry + (code_offset + operation.len) as u64;

                                let address = (next_rip as i64 + disp as i64) as u64;

                                let target_segment = segments
                                    .iter()
                                    .find(|segment| {
                                        segment.p_vaddr <= address
                                            && address < segment.p_vaddr + segment.p_filesz
                                    })
                                    .unwrap();

                                let target_file_offset =
                                    target_segment.p_offset + (address - target_segment.p_vaddr);

                                let data_start = target_file_offset as usize;
                                let data_end =
                                    (target_segment.p_offset + target_segment.p_filesz) as usize;

                                let data = &bytes[data_start..data_end];

                                dbg!(x86_register);
                                dbg!(address);
                                dbg!(target_file_offset);
                                dbg!(data);

                                riscv_data.extend_from_slice(data);

                                let fixup_index = riscv_code.len();

                                lea_fixups.push((fixup_index, x86_register.to_riscv(), address));

                                riscv_code.push(0);
                                riscv_code.push(0);

                                code_offset += operation.len;
                            }

                            X86operation::Xor => {
                                let mut modrm_offset = code_offset + 1;

                                if (code[code_offset] & 0xF0) == 0x40 {
                                    modrm_offset += 1;
                                }

                                let modrm = code[modrm_offset];

                                let mode = modrm >> 6;

                                match mode {
                                    0b11 => {
                                        let destination = X86Reg::from_modrm(modrm, false);
                                        let source = X86Reg::from_modrm_reg(modrm, false);

                                        riscv_code.push(encode::encode_xor(
                                            destination.to_riscv(),
                                            destination.to_riscv(),
                                            source.to_riscv(),
                                        ));

                                        registers[destination.to_index()] = 0;

                                        code_offset += operation.len;
                                    }

                                    _ => todo!(),
                                }
                            }

                            X86operation::Endbr64 => {
                                println!("WARNING: ENDBR64 is not translated at all");
                                code_offset += operation.len;
                            }

                            X86operation::Jmp => {
                                todo!();
                            }

                            X86operation::Call => {
                                todo!();
                            }

                            X86operation::Ret => {
                                todo!();
                            }

                            X86operation::ConditionalJump => {
                                todo!();
                            }

                            X86operation::Add => {
                                let mut pos = code_offset;

                                let mut rex = 0u8;
                                if (code[pos] & 0xF0) == 0x40 {
                                    rex = code[pos];
                                    pos += 1;
                                }

                                match code[pos] {
                                    0x01 => {
                                        let modrm = code[pos + 1];
                                        let mode = modrm >> 6;

                                        if mode != 0b11 {
                                            todo!();
                                        }

                                        let destination =
                                            X86Reg::from_modrm(modrm, (rex & 0b001) != 0);

                                        let source =
                                            X86Reg::from_modrm_reg(modrm, (rex & 0b100) != 0);

                                        let rd = destination.to_riscv();
                                        let rs1 = destination.to_riscv();
                                        let rs2 = source.to_riscv();

                                        riscv_code.push(encode::encode_add(rd, rs1, rs2));

                                        registers[destination.to_index()] = registers
                                            [destination.to_index()]
                                        .wrapping_add(registers[source.to_index()]);

                                        code_offset += operation.len;
                                    }

                                    _ => todo!(),
                                }
                            }

                            X86operation::Sub => {
                                let mut pos = code_offset;

                                let mut rex = 0u8;
                                if (code[pos] & 0xF0) == 0x40 {
                                    rex = code[pos];
                                    pos += 1;
                                }

                                match code[pos] {
                                    0x29 => {
                                        let modrm = code[pos + 1];
                                        let mode = modrm >> 6;

                                        if mode != 0b11 {
                                            todo!();
                                        }

                                        let destination =
                                            X86Reg::from_modrm(modrm, (rex & 0b001) != 0);

                                        let source =
                                            X86Reg::from_modrm_reg(modrm, (rex & 0b100) != 0);

                                        let rd = destination.to_riscv();
                                        let rs1 = destination.to_riscv();
                                        let rs2 = source.to_riscv();

                                        if (rex & 0b1000) != 0 {
                                            // 64-bit
                                            riscv_code.push(encode::encode_sub(rd, rs1, rs2));

                                            registers[destination.to_index()] = registers
                                                [destination.to_index()]
                                            .wrapping_sub(registers[source.to_index()]);
                                        } else {
                                            // 32-bit
                                            riscv_code.push(encode::encode_subw(rd, rs1, rs2));

                                            let result = (registers[destination.to_index()] as u32)
                                                .wrapping_sub(registers[source.to_index()] as u32);

                                            // x86-64 zapis do 32-bitowego rejestru zeruje górne 32 bity
                                            registers[destination.to_index()] = result as u64;
                                        }

                                        code_offset += operation.len;
                                    }

                                    _ => todo!(),
                                }
                            }

                            X86operation::And => {
                                let mut pos = code_offset;

                                let mut rex = 0u8;
                                if (code[pos] & 0xF0) == 0x40 {
                                    rex = code[pos];
                                    pos += 1;
                                }

                                match code[pos] {
                                    0x21 => {
                                        let modrm = code[pos + 1];
                                        let mode = modrm >> 6;

                                        if mode != 0b11 {
                                            todo!();
                                        }

                                        let destination =
                                            X86Reg::from_modrm(modrm, (rex & 0b001) != 0);

                                        let source =
                                            X86Reg::from_modrm_reg(modrm, (rex & 0b100) != 0);

                                        let rd = destination.to_riscv();
                                        let rs1 = destination.to_riscv();
                                        let rs2 = source.to_riscv();

                                        if (rex & 0b1000) != 0 {
                                            // 64-bit
                                            riscv_code.push(encode::encode_and(rd, rs1, rs2));

                                            registers[destination.to_index()] &=
                                                registers[source.to_index()];
                                        } else {
                                            // 32-bit
                                            riscv_code.push(encode::encode_andw(rd, rs1, rs2));

                                            let result = (registers[destination.to_index()] as u32)
                                                & (registers[source.to_index()] as u32);

                                            registers[destination.to_index()] = result as u64;
                                        }

                                        code_offset += operation.len;
                                    }

                                    0x83 => {
                                        let modrm = code[pos + 1];

                                        let mode = modrm >> 6;
                                        let reg = (modrm >> 3) & 0b111;

                                        if mode != 0b11 {
                                            todo!();
                                        }

                                        if reg != 0b100 {
                                            todo!();
                                        }

                                        let destination = X86Reg::from_modrm(modrm, false);
                                        let riscv_destination = destination.to_riscv();

                                        let imm = code[pos + 2] as i8 as i32;

                                        riscv_code.push(encode::encode_andi(
                                            riscv_destination,
                                            riscv_destination,
                                            imm,
                                        ));

                                        registers[destination.to_index()] &= imm as i64 as u64;

                                        code_offset += operation.len;
                                    }

                                    _ => todo!(),
                                }
                            }

                            X86operation::Or => {
                                let mut pos = code_offset;

                                let mut rex = 0u8;
                                if (code[pos] & 0xF0) == 0x40 {
                                    rex = code[pos];
                                    pos += 1;
                                }

                                match code[pos] {
                                    0x09 => {
                                        let modrm = code[pos + 1];
                                        let mode = modrm >> 6;

                                        if mode != 0b11 {
                                            todo!();
                                        }

                                        let destination =
                                            X86Reg::from_modrm(modrm, (rex & 0b001) != 0);

                                        let source =
                                            X86Reg::from_modrm_reg(modrm, (rex & 0b100) != 0);

                                        let rd = destination.to_riscv();
                                        let rs1 = destination.to_riscv();
                                        let rs2 = source.to_riscv();

                                        if (rex & 0b1000) != 0 {
                                            riscv_code.push(encode::encode_or(rd, rs1, rs2));

                                            registers[destination.to_index()] |=
                                                registers[source.to_index()];
                                        } else {
                                            riscv_code.push(encode::encode_orw(rd, rs1, rs2));

                                            let result = (registers[destination.to_index()] as u32)
                                                | (registers[source.to_index()] as u32);

                                            registers[destination.to_index()] = result as u64;
                                        }

                                        code_offset += operation.len;
                                    }

                                    _ => todo!(),
                                }
                            }

                            X86operation::Cmp => {
                                todo!();
                            }

                            X86operation::Test => {
                                todo!();
                            }

                            X86operation::Push => {
                                let x86_register = match code[code_offset] & 0b111 {
                                    0 => X86Reg::Rax,
                                    1 => X86Reg::Rcx,
                                    2 => X86Reg::Rdx,
                                    3 => X86Reg::Rbx,
                                    4 => X86Reg::Rsp,
                                    5 => X86Reg::Rbp,
                                    6 => X86Reg::Rsi,
                                    7 => X86Reg::Rdi,
                                    _ => unreachable!(),
                                };

                                let riscv_register = x86_register.to_riscv();

                                riscv_code.push(encode::encode_addi(2, 2, -8));

                                riscv_code.push(encode::encode_sd(riscv_register, 2, 0));

                                code_offset += operation.len;
                            }

                            X86operation::Pop => {
                                let x86_register = match code[code_offset] & 0b111 {
                                    0 => X86Reg::Rax,
                                    1 => X86Reg::Rcx,
                                    2 => X86Reg::Rdx,
                                    3 => X86Reg::Rbx,
                                    4 => X86Reg::Rsp,
                                    5 => X86Reg::Rbp,
                                    6 => X86Reg::Rsi,
                                    7 => X86Reg::Rdi,
                                    _ => unreachable!(),
                                };

                                let riscv_register = x86_register.to_riscv();

                                riscv_code.push(encode::encode_ld(riscv_register, 2, 0));

                                riscv_code.push(encode::encode_addi(2, 2, 8));

                                code_offset += operation.len;
                            }

                            X86operation::Nop => {
                                code_offset += operation.len;
                            }
                        }
                        println!("{:.2}%", code_offset as f64 / code.len() as f64 * 100.0);
                        if cfg!(debug_assertions) {
                            println!("{} out of {}", code_offset, code.len());
                        }
                    }
                }
            }

            let mut riscv_bytes = Vec::new();

            for (index, riscv_register, _address) in lea_fixups {
                let data_address = 0x11000u64;

                riscv_code[index] =
                    encode::encode_lui(riscv_register, ((data_address + 0x800) >> 12) as i32);

                riscv_code[index + 1] = encode::encode_addi(
                    riscv_register,
                    riscv_register,
                    (data_address as i64 & 0xfff) as i32,
                );
            }

            for instruction in &riscv_code {
                riscv_bytes.extend_from_slice(&instruction.to_le_bytes());
            }

            if cfg!(debug_assertions) {
                println!("RISCV CODE");

                for instruction in &riscv_code {
                    println!("{:08x}", instruction);
                }
            }

            elf::write_elf("Program".to_string(), riscv_bytes, riscv_data);

            let output = Command::new("llvm-objdump")
                .args(["-d", "-s", "Program"])
                .output()
                .unwrap();

            let mut asm = File::create("Program.asm").unwrap();
            asm.write_all(&output.stdout).unwrap();
        } else {
            panic!("Error reading file");
        }
    }
}
