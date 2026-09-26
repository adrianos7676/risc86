use crate::{
    CPUSTATEREG, JUNKREG0, encode,
    operation::handeler::{HandelerInputValue, HandelerReturnValue},
    translate::translate,
};

pub fn conditionaljump(values: HandelerInputValue<'_>) -> HandelerReturnValue {
    dbg!(values.code[values.code_offset]);

    match values.code[values.code_offset] {
        0x70 => todo!(), // JO
        0x71 => todo!(), // JNO
        0x72 => todo!(), // JB / JC
        0x73 => todo!(), // JAE / JNC

        // JE / JZ
        0x74 => {
            let displacement = values.code[values.code_offset + 1] as i8;

            let target =
                values.code_offset as i64 + values.operation.len as i64 + displacement as i64;

            values
                .riscv_code
                .push(encode::encode_andi(JUNKREG0, CPUSTATEREG, 0x40));

            values.riscv_code.push(encode::encode_bne(JUNKREG0, 0, 0));

            let branch_index = values.riscv_code.len() - 1;

            if target >= 0 && (target as usize) < values.code.len() {
                values.branch_fixups.push((branch_index, target as usize));

                let child_context = values.translation_context.clone();

                let translation_future = tokio::spawn(translate(
                    child_context,
                    target as usize,
                    values.registers.clone(),
                ));

                return HandelerReturnValue::future(values.operation.len, translation_future);
            }
        }

        0x75 => todo!(), // JNE / JNZ
        0x76 => todo!(), // JBE
        0x77 => todo!(), // JA
        0x78 => todo!(), // JS
        0x79 => todo!(), // JNS
        0x7A => todo!(), // JP / JPE
        0x7B => todo!(), // JNP / JPO
        0x7C => todo!(), // JL
        0x7D => todo!(), // JGE
        0x7E => todo!(), // JLE
        0x7F => todo!(), // JG

        // near JCC
        0x0F => {
            let opcode = values.code[values.code_offset + 1];

            match opcode {
                0x80..=0x8F => {
                    let condition = opcode & 0x0F;

                    let displacement = i32::from_le_bytes([
                        values.code[values.code_offset + 2],
                        values.code[values.code_offset + 3],
                        values.code[values.code_offset + 4],
                        values.code[values.code_offset + 5],
                    ]);

                    let target = values.code_offset as i64
                        + values.operation.len as i64
                        + displacement as i64;

                    values
                        .riscv_code
                        .push(encode::encode_andi(JUNKREG0, CPUSTATEREG, 0x40));

                    values.riscv_code.push(encode::encode_bne(JUNKREG0, 0, 0));

                    let branch_index = values.riscv_code.len() - 1;

                    if target >= 0 && (target as usize) < values.code.len() {
                        values.branch_fixups.push((branch_index, target as usize));

                        let child_context = values.translation_context.clone();

                        let translation_future = tokio::spawn(translate(
                            child_context,
                            target as usize,
                            values.registers.clone(),
                        ));

                        return HandelerReturnValue::future(
                            values.operation.len,
                            translation_future,
                        );
                    }

                    dbg!(condition, displacement);
                    dbg!(target);
                }

                _ => unreachable!(),
            }
        }

        _ => unreachable!(),
    }

    HandelerReturnValue::new(values.operation.len)
}
