
pub fn encode_addi(rd: u8, rs1: u8, imm: i32) -> u32 {
    ((imm as u32 & 0xfff) << 20)
        | ((rs1 as u32) << 15)
        | ((rd as u32) << 7)
        | 0x13
}

pub fn encode_ecall() -> u32 {
    0x00000073
}

pub fn encode_lui(rd: u8, imm: i32) -> u32 {
    ((imm as u32 & 0xfffff) << 12)
        | ((rd as u32) << 7)
        | 0x37
}