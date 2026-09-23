use crate::operation::handeler::{HandelerInputValue, HandelerReturnValue};

pub fn endbr64(values: HandelerInputValue) -> HandelerReturnValue {
    println!("WARNING: ENDBR64 is not translated at all");
    HandelerReturnValue {
        operation_len: values.operation.len,
    }
}
