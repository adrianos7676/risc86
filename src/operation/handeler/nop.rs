use crate::{encode::encode_addi, operation::handeler::{HandelerInputValue, HandelerReturnValue}};


pub fn nop(values: HandelerInputValue) -> HandelerReturnValue {
    values.riscv_code.push(encode_addi(0x0, 0x0, 0x0));
    HandelerReturnValue::new(values.operation.len)
}