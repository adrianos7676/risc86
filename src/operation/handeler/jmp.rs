use crate::{
    encode, operation::handeler::{HandelerInputValue, HandelerReturnValue}, translate::translate,
};

pub fn jmp(values: HandelerInputValue) -> HandelerReturnValue {
    let code = values.code;
    let code_offset = values.code_offset;

    let opcode = code[code_offset];

    let (instruction_len, displacement) = match opcode {
        0xE9 => {
            let displacement =
                i32::from_le_bytes(code[code_offset + 1..code_offset + 5].try_into().unwrap());

            (5, displacement as i64)
        }

        0xEB => {
            let displacement = code[code_offset + 1] as i8 as i64;

            (2, displacement)
        }

        _ => unreachable!(),
    };

    let target = (code_offset as i64 + instruction_len as i64 + displacement) as usize;

    let fixup_index = values.riscv_code.len();

    values.branch_fixups.push((fixup_index, target));

    values.riscv_code.push(encode::encode_jal(0, 0));

    let child_context = values.translation_context.clone();

    let translation_future = tokio::spawn(translate(
        child_context,
        target as usize,
        values.registers.clone(),
    ));

    HandelerReturnValue::finish_with_future(instruction_len, translation_future)
}
