use tokio::task::JoinHandle;
use std::sync::Arc;

use crate::{elf::{Elf64Header, Elf64ProgramHeader}, operation::DecodedInstruction, translate::TranslationResult};

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
pub mod cmp;
pub mod conditionaljump;
pub mod test;
pub mod jmp;
pub mod call;
pub mod ret;

#[derive(Clone)]
pub struct TranslationContext {
    pub code: Arc<[u8]>,
    pub header: Arc<Elf64Header>,
    pub bytes: Arc<Vec<u8>>,
    pub segments: Arc<Vec<Elf64ProgramHeader>>,
}

pub struct HandelerInputValue<'a> {
    pub code: &'a [u8],
    pub code_offset: usize,
    pub registers: &'a mut [u64; 16],
    pub operation: &'a DecodedInstruction,
    pub header: &'a Elf64Header,
    pub translation_context: &'a TranslationContext,
    pub bytes: &'a Vec<u8>,
    pub riscv_data: &'a mut Vec<u8>,
    pub riscv_code: &'a mut Vec<u32>,
    pub lea_fixups: &'a mut Vec<(usize, u8, u64)>,
    pub branch_fixups: &'a mut Vec<(usize, usize)>,
    pub call_fixups: &'a mut Vec<(usize, usize)>,
    pub segments: &'a Vec<crate::elf::Elf64ProgramHeader>,
}
pub struct HandelerReturnValue {
    pub operation_len: usize,
    pub finish_thread: bool,
    pub future: Option<JoinHandle<TranslationResult>>,
}

impl HandelerReturnValue {
    pub fn new(operation_len: usize) -> Self {
        Self {
            operation_len,
            finish_thread: false,
            future: None,
        }
    }

    pub fn future(operation_len: usize, future: JoinHandle<TranslationResult>) -> Self {
        Self {
            operation_len,
            finish_thread: false,
            future: Some(future),
        }
    }

    pub fn finish(operation_len: usize) -> Self {
        Self {
            operation_len,
            finish_thread: true,
            future: None,
        }
    }

    pub fn finish_with_future(operation_len: usize, future: JoinHandle<TranslationResult>) -> Self {
        Self {
            operation_len,
            finish_thread: true,
            future: Some(future),
        }
    }
}