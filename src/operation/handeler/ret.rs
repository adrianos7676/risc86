use crate::operation::handeler::{HandelerInputValue, HandelerReturnValue};

pub fn ret(values: HandelerInputValue) -> HandelerReturnValue {
    values.riscv_code.push(crate::encode::encode_jalr(0, 1, 0));

    HandelerReturnValue::finish(values.operation.len)
}