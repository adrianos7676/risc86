
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

pub fn encode_xor(rd: u8, rs1: u8, rs2: u8) -> u32 {
    ((rs2 as u32) << 20)
        | ((rs1 as u32) << 15)
        | (0b100 << 12)
        | ((rd as u32) << 7)
        | 0x33
}

pub fn encode_ld(rd: u8, rs1: u8, imm: i32) -> u32 {
    ((imm as u32 & 0xfff) << 20)
        | ((rs1 as u32) << 15)
        | (0b011 << 12)
        | ((rd as u32) << 7)
        | 0x03
}

pub fn encode_sd(rs2: u8, rs1: u8, imm: i32) -> u32 {
    let imm = imm as u32;

    ((imm >> 5 & 0x7f) << 25)
        | ((rs2 as u32) << 20)
        | ((rs1 as u32) << 15)
        | (0b011 << 12)
        | ((imm & 0x1f) << 7)
        | 0x23
}

pub fn encode_andi(rd: u8, rs1: u8, imm: i32) -> u32 {
    ((imm as u32 & 0xfff) << 20)
        | ((rs1 as u32) << 15)
        | (0b111 << 12)
        | ((rd as u32) << 7)
        | 0x13
}

pub fn encode_slli(rd: u8, rs1: u8, shamt: u8) -> u32 {
    ((shamt as u32) << 20)
        | ((rs1 as u32) << 15)
        | (0b001 << 12)
        | ((rd as u32) << 7)
        | 0x13
}

pub fn encode_add(rd: u8, rs1: u8, rs2: u8) -> u32 {
    ((rs2 as u32) << 20)
        | ((rs1 as u32) << 15)
        | ((rd as u32) << 7)
        | 0x33
}

pub fn encode_sub(rd: u8, rs1: u8, rs2: u8) -> u32 {
    ((rs2 as u32) << 20)
        | ((rs1 as u32) << 15)
        | (0b000u32 << 12)
        | ((rd as u32) << 7)
        | 0x33
}

pub fn encode_subw(rd: u8, rs1: u8, rs2: u8) -> u32 {
    ((rs2 as u32) << 20)
        | ((rs1 as u32) << 15)
        | (0b000u32 << 12)
        | ((rd as u32) << 7)
        | 0x3B
}

pub fn encode_and(rd: u8, rs1: u8, rs2: u8) -> u32 {
    ((rs2 as u32) << 20)
        | ((rs1 as u32) << 15)
        | (0b111 << 12)
        | ((rd as u32) << 7)
        | 0x33
}

pub fn encode_andw(rd: u8, rs1: u8, rs2: u8) -> u32 {
    ((rs2 as u32) << 20)
        | ((rs1 as u32) << 15)
        | (0b111 << 12)
        | ((rd as u32) << 7)
        | 0x3B
}

pub fn encode_or(rd: u8, rs1: u8, rs2: u8) -> u32 {
    ((rs2 as u32) << 20)
        | ((rs1 as u32) << 15)
        | (0b110 << 12)
        | ((rd as u32) << 7)
        | 0x33
}

pub fn encode_orw(rd: u8, rs1: u8, rs2: u8) -> u32 {
    ((rs2 as u32) << 20)
        | ((rs1 as u32) << 15)
        | (0b110 << 12)
        | ((rd as u32) << 7)
        | 0x3B
}