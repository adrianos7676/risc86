use crate::{
    CPUSTATEREG, JUNKREG0, JUNKREG1, X86Reg, encode,
    operation::handeler::{HandelerInputValue, HandelerReturnValue},
};

pub fn mul(values: HandelerInputValue) -> HandelerReturnValue {
    let mut pos = values.code_offset;

    let mut rex = 0u8;
    if (values.code[pos] & 0xF0) == 0x40 {
        rex = values.code[pos];
        pos += 1;
    }

    match values.code[pos] {
        0xF6 => {
            todo!()
        }
        0xF7 => {
            let modrm = values.code[pos + 1];
            let mode = modrm >> 6;

            match mode {
                0b11 => {
                    let is_64bit = rex & 0b1000 != 0;
                    let destination = X86Reg::Rax;

                    let source = X86Reg::from_modrm(modrm, (rex & 0b001) != 0);

                    if is_64bit {
                        values.riscv_code.push(encode::encode_and(
                            JUNKREG0,
                            destination.to_riscv(),
                            destination.to_riscv(),
                        ));

                        values.riscv_code.push(encode::encode_mul(
                            destination.to_riscv(),
                            JUNKREG0,
                            source.to_riscv(),
                        ));

                        values.riscv_code.push(encode::encode_mulhu(
                            X86Reg::Rdx.to_riscv(),
                            JUNKREG0,
                            source.to_riscv(),
                        ));
                    } else {
                        values.riscv_code.push(encode::encode_slli(
                            JUNKREG0,
                            destination.to_riscv(),
                            32,
                        ));
                        values
                            .riscv_code
                            .push(encode::encode_srli(JUNKREG0, JUNKREG0, 32));

                        values.riscv_code.push(encode::encode_slli(
                            JUNKREG1,
                            source.to_riscv(),
                            32,
                        ));
                        values
                            .riscv_code
                            .push(encode::encode_srli(JUNKREG1, JUNKREG1, 32));

                        values.riscv_code.push(encode::encode_mul(
                            destination.to_riscv(),
                            JUNKREG0,
                            JUNKREG1,
                        ));

                        values.riscv_code.push(encode::encode_srli(
                            X86Reg::Rdx.to_riscv(),
                            destination.to_riscv(),
                            32,
                        ));

                        values.riscv_code.push(encode::encode_slli(
                            destination.to_riscv(),
                            destination.to_riscv(),
                            32,
                        ));

                        values.riscv_code.push(encode::encode_srli(
                            destination.to_riscv(),
                            destination.to_riscv(),
                            32,
                        ));
                    }

                    // Clear old CF.
                    values
                        .riscv_code
                        .push(encode::encode_andi(CPUSTATEREG, CPUSTATEREG, -2));

                    // Clear old OF.
                    values
                        .riscv_code
                        .push(encode::encode_srli(JUNKREG0, CPUSTATEREG, 11));
                    values
                        .riscv_code
                        .push(encode::encode_andi(JUNKREG0, JUNKREG0, 1));
                    values
                        .riscv_code
                        .push(encode::encode_slli(JUNKREG0, JUNKREG0, 11));
                    values
                        .riscv_code
                        .push(encode::encode_sub(CPUSTATEREG, CPUSTATEREG, JUNKREG0));

                    // CF = RDX != 0
                    values.riscv_code.push(encode::encode_sltu(
                        JUNKREG0,
                        0,
                        X86Reg::Rdx.to_riscv(),
                    ));

                    // OF = CF
                    values
                        .riscv_code
                        .push(encode::encode_slli(JUNKREG1, JUNKREG0, 11));

                    // Set CF and OF.
                    values
                        .riscv_code
                        .push(encode::encode_or(JUNKREG0, JUNKREG0, JUNKREG1));
                    values
                        .riscv_code
                        .push(encode::encode_or(CPUSTATEREG, CPUSTATEREG, JUNKREG0));
                }
                _ => todo!(),
            }
        }
        _ => unreachable!(),
    }

    HandelerReturnValue::new(values.operation.len)
}
