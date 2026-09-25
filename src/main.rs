use std::{
    collections::HashMap, env, fs::File, io::Write, path::PathBuf, process::Command, sync::Arc,
};

use crate::operation::handeler::TranslationContext;

mod elf;
mod encode;
mod operation;
mod translate;

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

fn flatten_translation(
    result: translate::TranslationResult,
    riscv_code: &mut Vec<u32>,
    riscv_data: &mut Vec<u8>,
    lea_fixups: &mut Vec<(usize, u8, u64)>,
    jcc_fixups: &mut Vec<(usize, usize)>,
    address_map: &mut HashMap<usize, usize>,
) {
    let code_start = riscv_code.len();

    address_map.insert(result.code_address, code_start);

    for (offset, register, address) in result.lea_fixups {
        lea_fixups.push((
            code_start + offset,
            register,
            address,
        ));
    }

    for (branch_index, target_x86) in result.jcc_fixups {
        jcc_fixups.push((
            code_start + branch_index,
            target_x86,
        ));
    }

    riscv_code.extend(result.code);
    riscv_data.extend(result.data);

    for child in result.children {
        flatten_translation(
            child,
            riscv_code,
            riscv_data,
            lea_fixups,
            jcc_fixups,
            address_map,
        );
    }
}

#[tokio::main]
async fn main() {
    let mut riscv_code: Vec<u32> = Vec::new();
    let mut lea_fixups: Vec<(usize, u8, u64)> = Vec::new();
    let mut jcc_fixups: Vec<(usize, usize)> = Vec::new();
    let mut address_map = HashMap::new();

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
                panic!("Not an ELF64 file");
            }

            if &bytes[5] != &1 {
                panic!("Not little-endian");
            }

            let header = Arc::new(elf::Elf64Header::from_bytes(&bytes[0..64]));

            dbg!(&header);

            let mut segments: Vec<elf::Elf64ProgramHeader> = Vec::new();

            for i in 0..header.e_phnum {
                let offset = header.e_phoff as usize
                    + i as usize * header.e_phentsize as usize;

                let program_header =
                    elf::Elf64ProgramHeader::from_bytes(
                        &bytes[offset
                            ..offset + header.e_phentsize as usize],
                    );

                dbg!(&program_header);

                if program_header.p_type == 1 {
                    segments.push(program_header);
                }
            }

            let segments = Arc::new(segments);

            let mut riscv_data: Vec<u8> = Vec::new();

            for program_header in segments.iter() {
                if program_header.p_vaddr <= header.e_entry
                    && header.e_entry
                        < program_header.p_vaddr
                            + program_header.p_memsz
                {
                    let file_offset =
                        program_header.p_offset
                            + (header.e_entry
                                - program_header.p_vaddr);

                    let code_start = file_offset as usize;

                    let code_end =
                        (program_header.p_offset
                            + program_header.p_filesz)
                            as usize;

                    let code = &bytes[code_start..code_end];

                    dbg!(code);

                    let translation_context =
                        TranslationContext {
                            code: code.into(),
                            header: header.clone(),
                            bytes: bytes.clone().into(),
                            segments: segments.clone(),
                        };

                    let translation_result =
                        translate::translate(
                            translation_context,
                            0,
                            [0u64; 16],
                            0,
                        )
                        .await;

                    dbg!(&translation_result);

                    flatten_translation(
                        translation_result,
                        &mut riscv_code,
                        &mut riscv_data,
                        &mut lea_fixups,
                        &mut jcc_fixups,
                        &mut address_map,
                    );
                }
            }

            dbg!(&address_map);
            dbg!(&jcc_fixups);

            for (index, riscv_register, _address) in lea_fixups {
                let data_address = 0x11000u64;

                riscv_code[index] =
                    encode::encode_lui(
                        riscv_register,
                        ((data_address + 0x800) >> 12) as i32,
                    );

                riscv_code[index + 1] =
                    encode::encode_addi(
                        riscv_register,
                        riscv_register,
                        (data_address as i64 & 0xfff) as i32,
                    );
            }

            for (branch_index, target_x86) in jcc_fixups {
                let target_index = address_map[&target_x86];

                let branch_pc = branch_index * 4;
                let target_pc = target_index * 4;

                let offset =
                    target_pc as isize - branch_pc as isize;

                dbg!(
                    branch_index,
                    target_x86,
                    target_index,
                    offset
                );

                riscv_code[branch_index] =
                    encode::encode_bne(
                        JUNKREG0,
                        0,
                        offset as i32,
                    );
            }

            let mut riscv_bytes = Vec::new();

            for instruction in &riscv_code {
                riscv_bytes.extend_from_slice(
                    &instruction.to_le_bytes(),
                );
            }

            elf::write_elf(
                "Program".to_string(),
                riscv_bytes,
                riscv_data,
            );

            if cfg!(debug_assertions) {
                let output = Command::new("llvm-objdump")
                    .args(["-d", "-s", "Program"])
                    .output()
                    .unwrap();

                let mut asm =
                    File::create("Program.asm").unwrap();

                asm.write_all(&output.stdout).unwrap();
            }
        } else {
            panic!("Error reading file");
        }
    }
}