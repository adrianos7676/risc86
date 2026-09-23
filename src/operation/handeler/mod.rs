use crate::{elf::Elf64Header, operation::DecodedInstruction};

pub mod syscall;
pub mod mov;
pub mod lea;
pub mod xor;
pub mod endbr64;
pub mod add;
pub mod sub;
pub mod and;
pub mod or;
pub mod push;
pub mod pop;
pub mod nop;
pub struct HandelerInputValue<'a> {
    pub code: &'a [u8],
    pub code_offset: usize,
    pub registers: [u64; 16],
    pub riscv_code: &'a mut Vec<u32>,
    pub operation: &'a DecodedInstruction,
    pub header: &'a Elf64Header,
    pub bytes: &'a Vec<u8>,
    pub riscv_data: &'a mut Vec<u8>,
    pub lea_fixups: &'a mut Vec<(usize, u8, u64)>,
    pub segments: &'a Vec<crate::elf::Elf64ProgramHeader>,
}
pub struct HandelerReturnValue {
    pub operation_len: usize,
}