use std::{env, fs::File, io::Write, path::PathBuf, process::Command};

mod elf;
mod encode;
mod operation;

const CPUSTATEREG: u8 = 24;
const JUNKREG0: u8 = 25;
const JUNKREG1: u8 = 26;
const JUNKREG2: u8 = 27;

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

pub fn load_u64(riscv_register: u8, value: u64, riscv_code: &mut Vec<u32>) {
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

fn main() {
    let mut riscv_code: Vec<u32> = Vec::new();
    let registers: [u64; 16] = [0u64; 16];
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
                if &program_header.p_vaddr <= &header.e_entry
                    && &header.e_entry < &(program_header.p_vaddr + program_header.p_memsz)
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
                        let operation = operation::X86operation::decode(code, code_offset);
                        dbg!(&operation);

                        let handeler_input_value = operation::handeler::HandelerInputValue {
                            code: code,
                            code_offset: code_offset,
                            riscv_code: &mut riscv_code,
                            registers: registers,
                            operation: &operation,
                            header: &header,
                            bytes: &bytes,
                            riscv_data: &mut riscv_data,
                            lea_fixups: &mut lea_fixups,
                            segments: &segments,
                        };

                        let handler_return_value = match operation.operation {
                            operation::X86operation::Syscall => {
                                operation::handeler::syscall::syscall(handeler_input_value)
                            }

                            operation::X86operation::Mov => {
                                operation::handeler::mov::mov(handeler_input_value)
                            }

                            operation::X86operation::Lea => {
                                operation::handeler::lea::lea(handeler_input_value)
                            }

                            operation::X86operation::Xor => {
                                operation::handeler::xor::xor(handeler_input_value)
                            }

                            operation::X86operation::Endbr64 => {
                                operation::handeler::endbr64::endbr64(handeler_input_value)
                            }

                            operation::X86operation::Jmp => {
                                todo!();
                            }

                            operation::X86operation::Call => {
                                todo!();
                            }

                            operation::X86operation::Ret => {
                                todo!();
                            }

                            operation::X86operation::ConditionalJump => {
                                todo!();
                            }

                            operation::X86operation::Add => {
                                operation::handeler::add::add(handeler_input_value)
                            }

                            operation::X86operation::Sub => {
                                operation::handeler::sub::sub(handeler_input_value)
                            }

                            operation::X86operation::And => {
                                operation::handeler::and::and(handeler_input_value)
                            }

                            operation::X86operation::Or => {
                                operation::handeler::or::or(handeler_input_value)
                            }

                            operation::X86operation::Cmp => {
                                operation::handeler::cmp::cmp(handeler_input_value)
                            }

                            operation::X86operation::Test => {
                                todo!();
                            }

                            operation::X86operation::Push => {
                                operation::handeler::push::push(handeler_input_value)
                            }

                            operation::X86operation::Pop => {
                                operation::handeler::pop::pop(handeler_input_value)
                            }

                            operation::X86operation::Nop => {
                                operation::handeler::nop::nop(handeler_input_value)
                            }
                        };

                        code_offset += handler_return_value.operation_len;

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
