pub mod handeler;

#[derive(Debug)]
pub struct DecodedInstruction {
    pub operation: X86operation,
    pub len: usize,
}

impl X86operation {
    fn modrm_len(code: &[u8], offset: usize) -> usize {
        let modrm = code[offset];
        let mode = modrm >> 6;
        let rm = modrm & 0b111;

        let mut len = 1;

        if mode != 3 && rm == 4 {
            len += 1;

            let sib = code[offset + 1];
            let base = sib & 0b111;

            if mode == 0 && base == 5 {
                len += 4;
            }
        }

        match mode {
            0 => {
                if rm == 5 {
                    len += 4;
                }
            }
            1 => len += 1,
            2 => len += 4,
            _ => {}
        }

        len
    }
    pub fn decode(code: &[u8], offset: usize) -> DecodedInstruction {
        let mut pos = offset;
        let mut rex = false;

        dbg!(
            offset,
            code[offset],
            &code[offset.saturating_sub(10)..(offset + 10).min(code.len())]
        );

        if (code[pos] & 0xF0) == 0x40 {
            rex = true;
            pos += 1;
        }

        match code[pos] {
            0x0F => match code.get(pos + 1..) {
                Some([0x05, ..]) => DecodedInstruction {
                    operation: Self::Syscall,
                    len: (pos - offset) + 2,
                },
                _ => todo!(),
            },

            0xF3 => match code.get(pos + 1..) {
                Some([0x0F, 0x1E, 0xFA, ..]) => DecodedInstruction {
                    operation: Self::Endbr64,
                    len: (pos - offset) + 4,
                },
                _ => todo!(),
            },

            0x88..=0x8B => DecodedInstruction {
                operation: Self::Mov,
                len: (pos - offset) + 1 + Self::modrm_len(code, pos + 1),
            },

            0xC6..=0xC7 => DecodedInstruction {
                operation: Self::Mov,
                len: todo!(),
            },

            0xB8..=0xBF => DecodedInstruction {
                operation: Self::Mov,
                len: if rex { 10 } else { 5 },
            },

            0x8D => DecodedInstruction {
                operation: Self::Lea,
                len: (pos - offset) + 1 + Self::modrm_len(code, pos + 1),
            },

            0x00..=0x05 => DecodedInstruction {
                operation: Self::Add,
                len: match code[pos] {
                    0x00..=0x03 => (pos - offset) + 1 + Self::modrm_len(code, pos + 1),
                    0x04 => (pos - offset) + 2,
                    0x05 => (pos - offset) + 5,
                    _ => unreachable!(),
                },
            },

            0x28..=0x2D => DecodedInstruction {
                operation: Self::Sub,
                len: match code[pos] {
                    0x28..=0x2B => (pos - offset) + 1 + Self::modrm_len(code, pos + 1),
                    0x2C => (pos - offset) + 2,
                    0x2D => (pos - offset) + 5,
                    _ => unreachable!(),
                },
            },

            0x30..=0x33 => DecodedInstruction {
                operation: Self::Xor,
                len: (pos - offset) + 1 + Self::modrm_len(code, pos + 1),
            },

            0x34 => DecodedInstruction {
                operation: Self::Xor,
                len: (pos - offset) + 2,
            },

            0x35 => DecodedInstruction {
                operation: Self::Xor,
                len: (pos - offset) + 5,
            },

            0x20..=0x25 => DecodedInstruction {
                operation: Self::And,
                len: match code[pos] {
                    0x20..=0x23 => (pos - offset) + 1 + Self::modrm_len(code, pos + 1),
                    0x24 => (pos - offset) + 2,
                    0x25 => (pos - offset) + 5,
                    _ => unreachable!(),
                },
            },

            0x08..=0x0D => DecodedInstruction {
                operation: Self::Or,
                len: match code[pos] {
                    0x08..=0x0B => (pos - offset) + 1 + Self::modrm_len(code, pos + 1),
                    0x0C => (pos - offset) + 2,
                    0x0D => (pos - offset) + 5,
                    _ => unreachable!(),
                },
            },

            0x38..=0x3D => DecodedInstruction {
                operation: Self::Cmp,
                len: match code[pos] {
                    0x38..=0x3B => (pos - offset) + 1 + Self::modrm_len(code, pos + 1),
                    0x3C => (pos - offset) + 2,
                    0x3D => (pos - offset) + 5,
                    _ => unreachable!(),
                },
            },

            0x83 => DecodedInstruction {
                operation: Self::And,
                len: (pos - offset) + 1 + Self::modrm_len(code, pos + 1) + 1,
            },
            0x84..=0x85 => DecodedInstruction {
                operation: Self::Test,
                len: todo!(),
            },

            0x50..=0x57 => DecodedInstruction {
                operation: Self::Push,
                len: (pos - offset) + 1,
            },

            0x58..=0x5F => DecodedInstruction {
                operation: Self::Pop,
                len: (pos - offset) + 1,
            },

            0xE8 => DecodedInstruction {
                operation: Self::Call,
                len: (pos - offset) + 5,
            },

            0xC3 => DecodedInstruction {
                operation: Self::Ret,
                len: (pos - offset) + 1,
            },

            0xC2 => DecodedInstruction {
                operation: Self::Ret,
                len: (pos - offset) + 3,
            },

            0xE9 => DecodedInstruction {
                operation: Self::Jmp,
                len: (pos - offset) + 5,
            },

            0xEB => DecodedInstruction {
                operation: Self::Jmp,
                len: (pos - offset) + 2,
            },

            0x70..=0x7F => DecodedInstruction {
                operation: Self::ConditionalJump,
                len: (pos - offset) + 2,
            },

            0x90 => DecodedInstruction {
                operation: Self::Nop,
                len: (pos - offset) + 1,
            },

            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X86operation {
    Mov,
    Lea,
    Add,
    Sub,
    Xor,
    And,
    Or,
    Cmp,
    Test,
    Push,
    Pop,
    Call,
    Ret,
    Jmp,
    ConditionalJump,
    Syscall,
    Nop,
    Endbr64,
}