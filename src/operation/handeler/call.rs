use crate::{
    encode,
    operation::handeler::{HandelerInputValue, HandelerReturnValue},
    translate::translate,
};

pub fn call(values: HandelerInputValue<'_>) -> HandelerReturnValue {
    let displacement = i32::from_le_bytes(
        values.code[values.code_offset + 1..values.code_offset + 5]
            .try_into()
            .unwrap(),
    );

    let target =
        values.code_offset as i64 + values.operation.len as i64 + displacement as i64;

    let call_index = values.riscv_code.len();

    values.riscv_code.push(encode::encode_jal(1, 0));

    values
        .call_fixups
        .push((call_index, target as usize));

    let child_context = values.translation_context.clone();

    let translation_future = tokio::spawn(translate(
        child_context,
        target as usize,
        values.registers.clone(),
    ));

    HandelerReturnValue::future(
        values.operation.len,
        translation_future,
    )
}
