use crate::{
    encode,
    load_u64,
    operation::handeler::{HandelerInputValue, HandelerReturnValue},
    X86Reg,
};

pub fn mov(values: HandelerInputValue) -> HandelerReturnValue {
    let code = values.code;
    let code_offset = values.code_offset;
    let operation_len = values.operation.len;
    let riscv_code = values.riscv_code;

    match code[code_offset] {
        0xB8..=0xBF => {
            let rex_w = (code[code_offset] & 0x08) != 0;
            let rex_b = (code[code_offset] & 0x01) != 0;

            let opcode_offset = if (code[code_offset] & 0xF0) == 0x40 {
                code_offset + 1
            } else {
                code_offset
            };

            let opcode = code[opcode_offset];

            let mut register = opcode & 0b111;

            if rex_b {
                register += 8;
            }

            let x86_register = X86Reg::from_modrm(register, false);
            let riscv_register = x86_register.to_riscv();

            let value = if rex_w {
                u64::from_le_bytes(
                    code[opcode_offset + 1..opcode_offset + 9]
                        .try_into()
                        .unwrap(),
                )
            } else {
                u32::from_le_bytes(
                    code[opcode_offset + 1..opcode_offset + 5]
                        .try_into()
                        .unwrap(),
                ) as u64
            };

            if rex_w {
                load_u64(riscv_register, value, riscv_code);
            } else {
                riscv_code.push(encode::encode_addi(
                    riscv_register,
                    0,
                    value as i32,
                ));
            }

            HandelerReturnValue { operation_len }
        }

        0x48 => match code[code_offset + 1] {
            0xB8..=0xBF => {
                let opcode = code[code_offset + 1];

                let x86_register = match opcode & 0b111 {
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

                let value =
                    u64::from_le_bytes(code[code_offset + 2..code_offset + 10].try_into().unwrap());

                let riscv_register = x86_register.to_riscv();

                load_u64(riscv_register, value, riscv_code);

                HandelerReturnValue { operation_len }
            }

            0x89 => {
                let modrm = code[code_offset + 2];

                let destination = X86Reg::from_modrm(modrm, false);
                let source = X86Reg::from_modrm_reg(modrm, false);

                let riscv_destination = destination.to_riscv();
                let riscv_source = source.to_riscv();

                riscv_code.push(encode::encode_addi(
                    riscv_destination,
                    riscv_source,
                    0,
                ));

                HandelerReturnValue { operation_len }
            }

            0xC7 => {
                let modrm = code[code_offset + 2];

                let x86_register = X86Reg::from_modrm(modrm, false);
                let riscv_register = x86_register.to_riscv();

                let value =
                    u32::from_le_bytes(code[code_offset + 3..code_offset + 7].try_into().unwrap())
                        as u64;

                riscv_code.push(encode::encode_addi(
                    riscv_register,
                    0,
                    value as i32,
                ));

                HandelerReturnValue { operation_len }
            }

            _ => todo!(),
        },

        0x49 => match code[code_offset + 1] {
            0x89 => {
                let modrm = code[code_offset + 2];

                let destination = X86Reg::from_modrm(modrm, true);
                let source = X86Reg::from_modrm_reg(modrm, false);

                let riscv_destination = destination.to_riscv();
                let riscv_source = source.to_riscv();

                riscv_code.push(encode::encode_addi(
                    riscv_destination,
                    riscv_source,
                    0,
                ));

                HandelerReturnValue { operation_len }
            }

            _ => todo!(),
        },

        0x4C => match code[code_offset + 1] {
            0x89 => {
                let modrm = code[code_offset + 2];

                let destination = X86Reg::from_modrm(modrm, false);
                let source = X86Reg::from_modrm_reg(modrm, true);

                let riscv_destination = destination.to_riscv();
                let riscv_source = source.to_riscv();

                riscv_code.push(encode::encode_addi(
                    riscv_destination,
                    riscv_source,
                    0,
                ));

                HandelerReturnValue { operation_len }
            }

            _ => todo!(),
        },

        _ => todo!(),
    }
}