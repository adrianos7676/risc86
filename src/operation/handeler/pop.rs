use crate::{
    X86Reg, encode,
    operation::handeler::{HandelerInputValue, HandelerReturnValue},
};

pub fn pop(values: HandelerInputValue) -> HandelerReturnValue {
    let x86_register = match values.code[values.code_offset] & 0b111 {
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

    values
        .riscv_code
        .push(encode::encode_ld(riscv_register, 2, 0));

    values.riscv_code.push(encode::encode_addi(2, 2, 8));

    HandelerReturnValue {
        operation_len: values.operation.len,
    }
}
