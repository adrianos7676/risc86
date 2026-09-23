use crate::{X86Reg, encode, operation::handeler::{HandelerInputValue, HandelerReturnValue}};

pub fn sub(mut values: HandelerInputValue) -> HandelerReturnValue {
    let mut pos = values.code_offset;

    let mut rex = 0u8;
    if (values.code[pos] & 0xF0) == 0x40 {
        rex = values.code[pos];
        pos += 1;
    }

    match values.code[pos] {
        0x29 => {
            let modrm = values.code[pos + 1];
            let mode = modrm >> 6;

            if mode != 0b11 {
                todo!();
            }

            let destination = X86Reg::from_modrm(modrm, (rex & 0b001) != 0);

            let source = X86Reg::from_modrm_reg(modrm, (rex & 0b100) != 0);

            let rd = destination.to_riscv();
            let rs1 = destination.to_riscv();
            let rs2 = source.to_riscv();

            if (rex & 0b1000) != 0 {
                // 64-bit
                values.riscv_code.push(encode::encode_sub(rd, rs1, rs2));

                values.registers[destination.to_index()] = values.registers[destination.to_index()].wrapping_sub(values.registers[source.to_index()]);
            } else {
                // 32-bit
                values.riscv_code.push(encode::encode_subw(rd, rs1, rs2));

                let result = (values.registers[destination.to_index()] as u32)
                    .wrapping_sub(values.registers[source.to_index()] as u32);

                // x86-64 zero out the top 32 bits
                values.registers[destination.to_index()] = result as u64;
            }

            HandelerReturnValue { operation_len: values.operation.len }
        }

        _ => todo!(),
    }
}
