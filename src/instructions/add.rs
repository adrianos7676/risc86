use crate::{X86Reg, encode};

pub fn add_reg_to_reg(
    modrm: u8,
    riscv_code: &mut Vec<u32>,
) -> usize {
    match modrm >> 6 {
        0b11 => {
            let destination = X86Reg::from_modrm(modrm).to_riscv();
            let source = X86Reg::from_modrm_reg(modrm).to_riscv();

            riscv_code.push(encode::encode_add(destination, destination, source));

            3
        }
        0b10 => todo!(),
        0b01 => todo!(),
        0b00 => todo!(),
        _ => unreachable!(),
    }
}
