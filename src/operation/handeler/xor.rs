use crate::{
    X86Reg,
    encode,
    operation::handeler::{
        HandelerInputValue,
        HandelerReturnValue,
    },
};

pub fn xor(mut values: HandelerInputValue) -> HandelerReturnValue {
    let mut modrm_offset = values.code_offset + 1;
    let mut rex = 0;

    if (values.code[values.code_offset] & 0xF0) == 0x40 {
        rex = values.code[values.code_offset];
        modrm_offset += 1;
    }

    let modrm = values.code[modrm_offset];

    let mode = modrm >> 6;

    match mode {
        0b11 => {
            let destination =
                X86Reg::from_modrm(modrm, (rex & 0x01) != 0);

            let source =
                X86Reg::from_modrm_reg(modrm, (rex & 0x04) != 0);

            values.riscv_code.push(encode::encode_xor(
                destination.to_riscv(),
                destination.to_riscv(),
                source.to_riscv(),
            ));

            values.registers[destination.to_index()] = 0;

            HandelerReturnValue::new(values.operation.len)
        }

        _ => todo!(),
    }
}