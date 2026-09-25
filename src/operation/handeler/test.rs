use crate::{
    encode,
    operation::handeler::{HandelerInputValue, HandelerReturnValue},
    CPUSTATEREG, JUNKREG0, JUNKREG1, JUNKREG2, X86Reg,
};

pub fn test(values: HandelerInputValue) -> HandelerReturnValue {
    let code = values.code;
    let code_offset = values.code_offset;

    let mut modrm_offset = code_offset + 1;
    let mut rex = 0;

    if (code[code_offset] & 0xF0) == 0x40 {
        rex = code[code_offset];
        modrm_offset += 1;
    }

    let opcode = code[code_offset + (if rex != 0 { 1 } else { 0 })];
    let modrm = code[modrm_offset];
    let mode = modrm >> 6;

    match mode {
        0b11 => {
            let lhs = X86Reg::from_modrm(modrm, (rex & 0x01) != 0);
            let rhs = X86Reg::from_modrm_reg(modrm, (rex & 0x04) != 0);

            let operand_size = match opcode {
                0x84 => 8,
                0x85 => {
                    if (rex & 0x08) != 0 {
                        64
                    } else {
                        32
                    }
                }
                _ => unreachable!(),
            };

            values
                .riscv_code
                .push(encode::encode_xor(CPUSTATEREG, CPUSTATEREG, CPUSTATEREG));

            match operand_size {
                8 => {
                    values.riscv_code.push(encode::encode_and(
                        JUNKREG1,
                        lhs.to_riscv(),
                        rhs.to_riscv(),
                    ));

                    values
                        .riscv_code
                        .push(encode::encode_andi(JUNKREG1, JUNKREG1, 0xff));
                }

                32 => {
                    values.riscv_code.push(encode::encode_andw(
                        JUNKREG1,
                        lhs.to_riscv(),
                        rhs.to_riscv(),
                    ));
                }

                64 => {
                    values.riscv_code.push(encode::encode_and(
                        JUNKREG1,
                        lhs.to_riscv(),
                        rhs.to_riscv(),
                    ));
                }

                _ => unreachable!(),
            }

            // ZF
            values
                .riscv_code
                .push(encode::encode_sltu(JUNKREG0, 0, JUNKREG1));

            values
                .riscv_code
                .push(encode::encode_xori(JUNKREG0, JUNKREG0, 1));

            values
                .riscv_code
                .push(encode::encode_slli(JUNKREG0, JUNKREG0, 6));

            values
                .riscv_code
                .push(encode::encode_or(CPUSTATEREG, CPUSTATEREG, JUNKREG0));

            // SF
            let sign_bit = match operand_size {
                8 => 7,
                32 => 31,
                64 => 63,
                _ => unreachable!(),
            };

            values
                .riscv_code
                .push(encode::encode_srli(JUNKREG0, JUNKREG1, sign_bit));

            values
                .riscv_code
                .push(encode::encode_andi(JUNKREG0, JUNKREG0, 1));

            values
                .riscv_code
                .push(encode::encode_slli(JUNKREG0, JUNKREG0, 7));

            values
                .riscv_code
                .push(encode::encode_or(CPUSTATEREG, CPUSTATEREG, JUNKREG0));

            // PF
            values
                .riscv_code
                .push(encode::encode_andi(JUNKREG0, JUNKREG1, 0xff));

            values
                .riscv_code
                .push(encode::encode_srli(JUNKREG2, JUNKREG0, 4));

            values
                .riscv_code
                .push(encode::encode_xor(JUNKREG0, JUNKREG0, JUNKREG2));

            values
                .riscv_code
                .push(encode::encode_srli(JUNKREG2, JUNKREG0, 2));

            values
                .riscv_code
                .push(encode::encode_xor(JUNKREG0, JUNKREG0, JUNKREG2));

            values
                .riscv_code
                .push(encode::encode_srli(JUNKREG2, JUNKREG0, 1));

            values
                .riscv_code
                .push(encode::encode_xor(JUNKREG0, JUNKREG0, JUNKREG2));

            values
                .riscv_code
                .push(encode::encode_andi(JUNKREG0, JUNKREG0, 1));

            values
                .riscv_code
                .push(encode::encode_xori(JUNKREG0, JUNKREG0, 1));

            values
                .riscv_code
                .push(encode::encode_slli(JUNKREG0, JUNKREG0, 2));

            values
                .riscv_code
                .push(encode::encode_or(CPUSTATEREG, CPUSTATEREG, JUNKREG0));

            HandelerReturnValue::new(values.operation.len)
        }

        _ => todo!(),
    }
}
