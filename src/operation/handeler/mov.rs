use crate::{
    X86Reg, encode, load_u64,
    operation::handeler::{HandelerInputValue, HandelerReturnValue},
};

pub fn mov(values: HandelerInputValue) -> HandelerReturnValue {
    let code = values.code;
    let code_offset = values.code_offset;
    let operation_len = values.operation.len;
    let riscv_code = values.riscv_code;

    match code[code_offset] {
        // mov r32, imm32
        0xB8..=0xBF => {
            let opcode = code[code_offset];

            let register = opcode & 0b111;

            let x86_register = X86Reg::from_modrm(register, false);
            let riscv_register = x86_register.to_riscv();

            let value =
                u32::from_le_bytes(code[code_offset + 1..code_offset + 5].try_into().unwrap())
                    as u64;

            values.registers[x86_register.to_index()] = value;

            riscv_code.push(encode::encode_addi(riscv_register, 0, value as i32));

            HandelerReturnValue::new(operation_len)
        }

        // REX.W / REX.W + REX.B
        0x48 | 0x49 => match code[code_offset + 1] {
            // mov r64, imm64
            0xB8..=0xBF => {
                let opcode = code[code_offset + 1];

                let register = opcode & 0b111;
                let rex_b = code[code_offset] & 0x01 != 0;

                let x86_register = X86Reg::from_modrm(register, rex_b);
                let riscv_register = x86_register.to_riscv();

                let value =
                    u64::from_le_bytes(code[code_offset + 2..code_offset + 10].try_into().unwrap());

                values.registers[x86_register.to_index()] = value;

                load_u64(riscv_register, value, riscv_code);

                HandelerReturnValue::new(operation_len)
            }

            // mov r/m64, r64
            0x89 => {
                let modrm = code[code_offset + 2];

                let destination = X86Reg::from_modrm(modrm, code[code_offset] & 0x01 != 0);

                let source = X86Reg::from_modrm_reg(modrm, false);

                let riscv_destination = destination.to_riscv();
                let riscv_source = source.to_riscv();

                values.registers[destination.to_index()] = values.registers[source.to_index()];

                riscv_code.push(encode::encode_addi(riscv_destination, riscv_source, 0));

                HandelerReturnValue::new(operation_len)
            }

            _ => todo!(),
        },

        // REX.W + REX.R
        0x4C => match code[code_offset + 1] {
            // mov r/m64, r64
            0x89 => {
                let modrm = code[code_offset + 2];

                let destination = X86Reg::from_modrm(modrm, false);
                let source = X86Reg::from_modrm_reg(modrm, true);

                let riscv_destination = destination.to_riscv();
                let riscv_source = source.to_riscv();

                values.registers[destination.to_index()] = values.registers[source.to_index()];

                riscv_code.push(encode::encode_addi(riscv_destination, riscv_source, 0));

                HandelerReturnValue::new(operation_len)
            }

            _ => todo!(),
        },

        _ => todo!(),
    }
}
