use crate::{
    CPUSTATEREG, JUNKREG0, JUNKREG1, JUNKREG2, X86Reg, encode, operation::handeler::{HandelerInputValue, HandelerReturnValue},
};

pub fn cmp(values: HandelerInputValue) -> HandelerReturnValue {
    let code = values.code;
    let code_offset = values.code_offset;

    let mut modrm_offset = code_offset + 1;
    let mut rex = 0;

    if (code[code_offset] & 0xF0) == 0x40 {
        rex = code[code_offset];
        modrm_offset += 1;
    }

    let modrm = code[modrm_offset];
    let mode = modrm >> 6;

    match mode {
        0b11 => {
            let lhs = X86Reg::from_modrm(modrm, (rex & 0x01) != 0);
            let rhs = X86Reg::from_modrm_reg(modrm, (rex & 0x04) != 0);
            let operand_size = if (rex & 0x08) != 0 { 64 } else { 32 };

            // Clear CPUSTATEREG (x24).
            // This resets all previously stored flags.
            values
                .riscv_code
                .push(encode::encode_xor(CPUSTATEREG, CPUSTATEREG, CPUSTATEREG));

            // Calculate Carry Flag (CF).
            //
            // JUNKREG0 = 1 if lhs < rhs (unsigned)
            //           0 otherwise
            //
            // For x86 SUB/CMP:
            //     CF = lhs < rhs
            values.riscv_code.push(encode::encode_sltu(
                JUNKREG0,
                lhs.to_riscv(),
                rhs.to_riscv(),
            ));

            // Store CF in bit 0 of CPUSTATEREG.
            //
            // CPUSTATEREG |= CF
            values
                .riscv_code
                .push(encode::encode_or(CPUSTATEREG, CPUSTATEREG, JUNKREG0));

            // Calculate the subtraction result.
            //
            // CMP does not store this result in an x86 register,
            // but we need it to calculate the remaining flags.
            //
            // JUNKREG1 = lhs - rhs
            if operand_size == 64 {
                values.riscv_code.push(encode::encode_sub(
                    JUNKREG1,
                    lhs.to_riscv(),
                    rhs.to_riscv(),
                ));
            } else {
                values.riscv_code.push(encode::encode_subw(
                    JUNKREG1,
                    lhs.to_riscv(),
                    rhs.to_riscv(),
                ));
            }

            // Check whether the subtraction result is non-zero.
            //
            // x0 is always zero.
            //
            // JUNKREG0 = 1 if JUNKREG1 != 0
            //            0 if JUNKREG1 == 0
            //
            // This temporarily gives us:
            //     JUNKREG0 = (result != 0)
            values
                .riscv_code
                .push(encode::encode_sltu(JUNKREG0, 0, JUNKREG1));

            // Invert the result.
            //
            // 0 XOR 1 = 1
            // 1 XOR 1 = 0
            //
            // Now:
            //     JUNKREG0 = 1 if result == 0
            //            = 0 otherwise
            //
            // Therefore:
            //     JUNKREG0 = ZF
            values
                .riscv_code
                .push(encode::encode_xori(JUNKREG0, JUNKREG0, 1));

            // Move ZF to bit 6.
            //
            // x86 RFLAGS:
            //     bit 6 = ZF
            //
            // 1 << 6 = 0b01000000
            values
                .riscv_code
                .push(encode::encode_slli(JUNKREG0, JUNKREG0, 6));

            // Store ZF in CPUSTATEREG.
            //
            // CPUSTATEREG |= ZF
            values
                .riscv_code
                .push(encode::encode_or(CPUSTATEREG, CPUSTATEREG, JUNKREG0));

            // Extract the sign bit from the subtraction result.
            // Bit 63 is the sign bit for 64-bit operands,
            // while bit 31 is the sign bit for 32-bit operands.
            values.riscv_code.push(encode::encode_srli(
                JUNKREG0,
                JUNKREG1,
                if operand_size == 64 { 63 } else { 31 },
            ));

            // Move SF from bit 0 to bit 7 in RFLAGS.
            //
            // x86 RFLAGS:
            //     bit 7 = SF
            values
                .riscv_code
                .push(encode::encode_slli(JUNKREG0, JUNKREG0, 7));

            // Merge SF into the emulated RFLAGS register.
            values
                .riscv_code
                .push(encode::encode_or(CPUSTATEREG, CPUSTATEREG, JUNKREG0));

            // Calculate Auxiliary Carry Flag (AF).
            //
            // AF is bit 4 of:
            //     lhs XOR rhs XOR result
            values
                .riscv_code
                .push(encode::encode_xor(JUNKREG0, lhs.to_riscv(), rhs.to_riscv()));

            values
                .riscv_code
                .push(encode::encode_xor(JUNKREG0, JUNKREG0, JUNKREG1));

            // Extract bit 4.
            values
                .riscv_code
                .push(encode::encode_srli(JUNKREG0, JUNKREG0, 4));

            values
                .riscv_code
                .push(encode::encode_andi(JUNKREG0, JUNKREG0, 1));

            // Move AF to bit 4 in RFLAGS.
            values
                .riscv_code
                .push(encode::encode_slli(JUNKREG0, JUNKREG0, 4));

            // Merge AF into RFLAGS.
            values
                .riscv_code
                .push(encode::encode_or(CPUSTATEREG, CPUSTATEREG, JUNKREG0));

            // Calculate Overflow Flag (OF).
            //
            // For subtraction:
            //     OF = ((lhs XOR rhs) AND (lhs XOR result)) >> sign_bit
            values
                .riscv_code
                .push(encode::encode_xor(JUNKREG0, lhs.to_riscv(), rhs.to_riscv()));

            values
                .riscv_code
                .push(encode::encode_xor(JUNKREG2, lhs.to_riscv(), JUNKREG1));

            values
                .riscv_code
                .push(encode::encode_and(JUNKREG0, JUNKREG0, JUNKREG2));

            // Extract the sign bit.
            values.riscv_code.push(encode::encode_srli(
                JUNKREG0,
                JUNKREG0,
                if operand_size == 64 { 63 } else { 31 },
            ));

            // Move OF to bit 11 in RFLAGS.
            values
                .riscv_code
                .push(encode::encode_slli(JUNKREG0, JUNKREG0, 11));

            // Merge OF into RFLAGS.
            values
                .riscv_code
                .push(encode::encode_or(CPUSTATEREG, CPUSTATEREG, JUNKREG0));

            // Calculate Parity Flag (PF).
            //
            // PF is based only on the low 8 bits of the result.
            // PF = 1 when the number of set bits is even.
            values
                .riscv_code
                .push(encode::encode_andi(JUNKREG0, JUNKREG1, 0xFF));

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

            // XOR gives 1 for odd parity.
            // x86 PF uses the opposite convention: 1 for even parity.
            values
                .riscv_code
                .push(encode::encode_xori(JUNKREG0, JUNKREG0, 1));

            // Move PF to bit 2 in RFLAGS.
            values
                .riscv_code
                .push(encode::encode_slli(JUNKREG0, JUNKREG0, 2));

            // Merge PF into RFLAGS.
            values
                .riscv_code
                .push(encode::encode_or(CPUSTATEREG, CPUSTATEREG, JUNKREG0));

            dbg!(
                code_offset,
                &code[code_offset..modrm_offset + 1],
                rex,
                modrm,
                mode,
                modrm & 0b111,
                (modrm >> 3) & 0b111,
                lhs,
                rhs,
                operand_size
            );

            HandelerReturnValue::new(modrm_offset - code_offset + 1)
        }

        _ => todo!(),
    }
}
