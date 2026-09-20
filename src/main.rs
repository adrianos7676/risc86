use std::{env, path::PathBuf};

mod encode;
mod elf;

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

impl X86Reg {
    fn from_modrm(byte: u8) -> Self {
        match byte & 0b111 {
            0 => Self::Rax,
            1 => Self::Rcx,
            2 => Self::Rdx,
            3 => Self::Rbx,
            4 => Self::Rsp,
            5 => Self::Rbp,
            6 => Self::Rsi,
            7 => Self::Rdi,
            _ => unreachable!(),
        }
    }

    fn from_modrm_reg(byte: u8) -> Self {
        match (byte >> 3) & 0b111 {
            0 => Self::Rax,
            1 => Self::Rcx,
            2 => Self::Rdx,
            3 => Self::Rbx,
            4 => Self::Rsp,
            5 => Self::Rbp,
            6 => Self::Rsi,
            7 => Self::Rdi,
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
    Exit
}

fn main() {
    let mut riscv_code: Vec<u32> = Vec::new();
    let mut registers = [0u64; 16];
    let mut lea_fixup: Option<(usize, u8)> = None;

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
                let offset =
                    header.e_phoff as usize +
                    i as usize * header.e_phentsize as usize;

                let program_header =
                    elf::Elf64ProgramHeader::from_bytes(
                        &bytes[offset..offset + header.e_phentsize as usize]
                    );

                dbg!(&program_header);
                
                if program_header.p_type == 1 {
                    segments.push(program_header);
                }
            }

            let mut riscv_data: Vec<u8> = Vec::new();

            for program_header in segments.iter() {
                if program_header.p_vaddr <= header.e_entry && header.e_entry < program_header.p_vaddr + program_header.p_memsz {
                    let file_offset = program_header.p_offset + (header.e_entry - program_header.p_vaddr);

                    let code_start = file_offset as usize;
                    let code_end = (program_header.p_offset + program_header.p_filesz) as usize;

                    let code = &bytes[code_start..code_end];

                    dbg!(code);

                    let mut code_offset = 0;
                    while code_offset < code.len() {
                        match code[code_offset] {
                            0x0F => {
                                match code[code_offset + 1] {
                                    //SYSCALL
                                    0x05 => {
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


                                        riscv_code.push(
                                            encode::encode_addi(
                                                17,
                                                0,
                                                riscv_syscall as i32
                                            )
                                        );

                                        riscv_code.push(
                                            encode::encode_ecall()
                                        );

                                        code_offset += 2;
                                    }
                                    _ => todo!()
                                }
                            },
                            0x48 => {
                                match code[code_offset + 1] {
                                    //MOV
                                    0xC7 => {
                                        let modrm = code[code_offset + 2];
                                        let x86_register = X86Reg::from_modrm(modrm);
                                        let riscv_register = x86_register.to_riscv();
                                        let value = u32::from_le_bytes(code[code_offset + 3..code_offset + 7].try_into().unwrap()) as u64;

                                        registers[x86_register.to_index()] = value;

                                        dbg!(riscv_register);
                                        dbg!(value);
                                        dbg!(registers);

                                        riscv_code.push(encode::encode_addi(riscv_register, 0, value as i32));
                                        code_offset = code_offset + 7;
                                    },
                                    //LEA
                                    0x8D => {
                                        let modrm = code[code_offset + 2];
                                        let x86_register = X86Reg::from_modrm_reg(modrm);

                                        let disp = i32::from_le_bytes(
                                            code[code_offset + 3..code_offset + 7]
                                                .try_into()
                                                .unwrap()
                                        );

                                        let next_rip =
                                            header.e_entry + (code_offset + 7) as u64;

                                        let address =
                                            (next_rip as i64 + disp as i64) as u64;

                                        let target_segment = segments
                                            .iter()
                                            .find(|segment|
                                                segment.p_vaddr <= address
                                                    && address < segment.p_vaddr + segment.p_filesz
                                            )
                                            .unwrap();

                                        let target_file_offset =
                                            target_segment.p_offset
                                                + (address - target_segment.p_vaddr);

                                        let data_start = target_file_offset as usize;
                                        let data_end =
                                            (target_segment.p_offset + target_segment.p_filesz) as usize;

                                        let data = &bytes[data_start..data_end];

                                        dbg!(x86_register);
                                        dbg!(address);
                                        dbg!(target_file_offset);
                                        dbg!(data);

                                        riscv_data.extend_from_slice(data);

                                        lea_fixup = Some((riscv_code.len(), x86_register.to_riscv()));

                                        riscv_code.push(0);
                                        riscv_code.push(0);

                                        code_offset += 7;
                                    }
                                    //XOR
                                    0x31 => {
                                        let modrm = code[code_offset + 1];
                                        let mode = modrm >> 6;
                                        match mode {
                                            0b11 => {
                                                let destination = X86Reg::from_modrm(modrm);
                                                let source = X86Reg::from_modrm_reg(modrm);

                                                todo!()
                                            }
                                            _ => todo!()
                                        }
                                    }
                                    _ => {
                                        dbg!(code[code_offset + 1]);
                                        todo!()
                                    }
                                }   
                            },
                            _ => todo!()
                        }
                    }
                }
            }

            let mut riscv_bytes = Vec::new();

            if let Some((index, riscv_register)) = lea_fixup {
                let data_address =
                    0x10000 + (riscv_code.len() * 4) as u64;

                riscv_code[index] =
                    encode::encode_lui(
                        riscv_register,
                        ((data_address + 0x800) >> 12) as i32
                    );

                riscv_code[index + 1] =
                    encode::encode_addi(
                        riscv_register,
                        riscv_register,
                        (data_address as i64 & 0xfff) as i32
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
        } else {
            panic!("Error reading file");
        }
    }
}
