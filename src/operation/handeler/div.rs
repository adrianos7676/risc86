use crate::{
    JUNKREG0, JUNKREG1, JUNKREG2, X86Reg, encode,
    operation::handeler::{HandelerInputValue, HandelerReturnValue},
};

fn emit_divide_error(values: &mut HandelerInputValue) {
    values.riscv_code.push(encode::encode_jal(0, 0));
}

fn emit_far_jump_placeholder(code: &mut Vec<u32>) -> usize {
    let jump_at = code.len();

    code.push(encode::encode_addi(0, 0, 0));
    code.push(encode::encode_addi(0, 0, 0));

    jump_at
}

fn patch_far_jump(
    code: &mut Vec<u32>,
    jump_at: usize,
    target: usize,
) {
    let offset =
        ((target as isize - jump_at as isize) * 4) as i64;

    let hi20 =
        (offset + 0x800) >> 12;

    let lo12 =
        offset - (hi20 << 12);

    code[jump_at] =
        encode::encode_auipc(
            JUNKREG2,
            hi20 as i32,
        );

    code[jump_at + 1] =
        encode::encode_jalr(
            0,
            JUNKREG2,
            lo12 as i32,
        );
}

fn patch_far_branch(
    code: &mut Vec<u32>,
    branch_at: usize,
    jump_at: usize,
    target: usize,
    branch: fn(u8, u8, i32) -> u32,
    rs1: u8,
    rs2: u8,
) {
    let skip_offset =
        ((jump_at as isize - branch_at as isize) * 4) as i32;

    code[branch_at] =
        branch(
            rs1,
            rs2,
            skip_offset,
        );

    patch_far_jump(
        code,
        jump_at,
        target,
    );
}

pub fn div(mut values: HandelerInputValue) -> HandelerReturnValue {
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
                    let is_64bit =
                        rex & 0x08 != 0;

                    let source =
                        X86Reg::from_modrm(
                            modrm,
                            (rex & 0x01) != 0,
                        );

                    let rax =
                        X86Reg::Rax.to_riscv();

                    let rdx =
                        X86Reg::Rdx.to_riscv();

                    let source_reg =
                        source.to_riscv();

                    if is_64bit {
                        /*
                         * Check divisor == 0.
                         *
                         * JUNKREG1 = divisor
                         * JUNKREG0 = divisor != 0
                         */
                        values
                            .riscv_code
                            .push(encode::encode_addi(
                                JUNKREG1,
                                source_reg,
                                0,
                            ));

                        values
                            .riscv_code
                            .push(encode::encode_sltu(
                                JUNKREG0,
                                JUNKREG1,
                                1,
                            ));

                        let divisor_zero_branch =
                            values.riscv_code.len();

                        values
                            .riscv_code
                            .push(encode::encode_bne(
                                JUNKREG0,
                                0,
                                0,
                            ));

                        let divisor_zero_jump =
                            emit_far_jump_placeholder(
                                &mut values.riscv_code,
                            );

                        /*
                         * Overflow condition:
                         *
                         * if RDX >= divisor:
                         *     #DE
                         *
                         * JUNKREG0 = RDX < divisor
                         */
                        values
                            .riscv_code
                            .push(encode::encode_sltu(
                                JUNKREG0,
                                rdx,
                                JUNKREG1,
                            ));

                        let overflow_branch =
                            values.riscv_code.len();

                        values
                            .riscv_code
                            .push(encode::encode_beq(
                                JUNKREG0,
                                0,
                                0,
                            ));

                        let overflow_jump =
                            emit_far_jump_placeholder(
                                &mut values.riscv_code,
                            );

                        /*
                         * If RDX == 0 we can use hardware DIVU.
                         *
                         * Otherwise use the software 128/64
                         * division path.
                         */
                        let slow_branch =
                            values.riscv_code.len();

                        values
                            .riscv_code
                            .push(encode::encode_bne(
                                rdx,
                                0,
                                0,
                            ));

                        /*
                         * Fast path:
                         *
                         * quotient = RAX / divisor
                         * remainder = RAX % divisor
                         */
                        values
                            .riscv_code
                            .push(encode::encode_remu(
                                JUNKREG0,
                                rax,
                                JUNKREG1,
                            ));

                        values
                            .riscv_code
                            .push(encode::encode_divu(
                                rax,
                                rax,
                                JUNKREG1,
                            ));

                        values
                            .riscv_code
                            .push(encode::encode_addi(
                                rdx,
                                JUNKREG0,
                                0,
                            ));

                        let fast_end_jump =
                            emit_far_jump_placeholder(
                                &mut values.riscv_code,
                            );

                        /*
                         * Slow path.
                         *
                         * Divide the 128-bit value
                         *
                         *     RDX:RAX
                         *
                         * by the 64-bit divisor.
                         */
                        let slow_start =
                            values.riscv_code.len();

                        values
                            .riscv_code
                            .push(encode::encode_addi(
                                JUNKREG1,
                                source_reg,
                                0,
                            ));

                        values
                            .riscv_code
                            .push(encode::encode_addi(
                                JUNKREG2,
                                rax,
                                0,
                            ));

                        values
                            .riscv_code
                            .push(encode::encode_addi(
                                JUNKREG0,
                                0,
                                0,
                            ));

                        for _ in 0..64 {
                            /*
                             * Carry from RDX.
                             */
                            values
                                .riscv_code
                                .push(encode::encode_srli(
                                    rax,
                                    rdx,
                                    63,
                                ));

                            let carry_branch =
                                values.riscv_code.len();

                            values
                                .riscv_code
                                .push(encode::encode_bne(
                                    rax,
                                    0,
                                    0,
                                ));

                            /*
                             * Normal path.
                             */
                            values
                                .riscv_code
                                .push(encode::encode_srli(
                                    rax,
                                    JUNKREG2,
                                    63,
                                ));

                            values
                                .riscv_code
                                .push(encode::encode_slli(
                                    JUNKREG2,
                                    JUNKREG2,
                                    1,
                                ));

                            values
                                .riscv_code
                                .push(encode::encode_slli(
                                    rdx,
                                    rdx,
                                    1,
                                ));

                            values
                                .riscv_code
                                .push(encode::encode_or(
                                    rdx,
                                    rdx,
                                    rax,
                                ));

                            values
                                .riscv_code
                                .push(encode::encode_slli(
                                    JUNKREG0,
                                    JUNKREG0,
                                    1,
                                ));

                            values
                                .riscv_code
                                .push(encode::encode_sltu(
                                    rax,
                                    rdx,
                                    JUNKREG1,
                                ));

                            let no_subtract =
                                values.riscv_code.len();

                            values
                                .riscv_code
                                .push(encode::encode_bne(
                                    rax,
                                    0,
                                    0,
                                ));

                            values
                                .riscv_code
                                .push(encode::encode_sub(
                                    rdx,
                                    rdx,
                                    JUNKREG1,
                                ));

                            values
                                .riscv_code
                                .push(encode::encode_ori(
                                    JUNKREG0,
                                    JUNKREG0,
                                    1,
                                ));

                            let skip_carry =
                                values.riscv_code.len();

                            values
                                .riscv_code
                                .push(encode::encode_jal(
                                    0,
                                    0,
                                ));

                            let carry_path =
                                values.riscv_code.len();

                            /*
                             * Carry path.
                             */
                            values
                                .riscv_code
                                .push(encode::encode_srli(
                                    rax,
                                    JUNKREG2,
                                    63,
                                ));

                            values
                                .riscv_code
                                .push(encode::encode_slli(
                                    JUNKREG2,
                                    JUNKREG2,
                                    1,
                                ));

                            values
                                .riscv_code
                                .push(encode::encode_slli(
                                    rdx,
                                    rdx,
                                    1,
                                ));

                            values
                                .riscv_code
                                .push(encode::encode_or(
                                    rdx,
                                    rdx,
                                    rax,
                                ));

                            values
                                .riscv_code
                                .push(encode::encode_slli(
                                    JUNKREG0,
                                    JUNKREG0,
                                    1,
                                ));

                            values
                                .riscv_code
                                .push(encode::encode_sub(
                                    rdx,
                                    rdx,
                                    JUNKREG1,
                                ));

                            values
                                .riscv_code
                                .push(encode::encode_ori(
                                    JUNKREG0,
                                    JUNKREG0,
                                    1,
                                ));

                            let iteration_end =
                                values.riscv_code.len();

                            /*
                             * carry branch
                             */
                            let carry_offset =
                                ((carry_path as isize
                                    - carry_branch as isize)
                                    * 4)
                                    as i32;

                            values.riscv_code[carry_branch] =
                                encode::encode_bne(
                                    rax,
                                    0,
                                    carry_offset,
                                );

                            /*
                             * no subtract branch
                             */
                            let no_subtract_offset =
                                ((skip_carry as isize
                                    - no_subtract as isize)
                                    * 4)
                                    as i32;

                            values.riscv_code[no_subtract] =
                                encode::encode_bne(
                                    rax,
                                    0,
                                    no_subtract_offset,
                                );

                            /*
                             * Skip carry path.
                             */
                            let skip_carry_offset =
                                ((iteration_end as isize
                                    - skip_carry as isize)
                                    * 4)
                                    as i32;

                            values.riscv_code[skip_carry] =
                                encode::encode_jal(
                                    0,
                                    skip_carry_offset,
                                );
                        }

                        values
                            .riscv_code
                            .push(encode::encode_addi(
                                rax,
                                JUNKREG0,
                                0,
                            ));

                        let normal_end =
                            values.riscv_code.len();

                        let end_jump =
                            emit_far_jump_placeholder(
                                &mut values.riscv_code,
                            );

                        /*
                         * Divide error handler.
                         */
                        let error_start =
                            values.riscv_code.len();

                        emit_divide_error(
                            &mut values,
                        );

                        let after_error =
                            values.riscv_code.len();

                        /*
                         * Patch divisor == 0.
                         */
                        patch_far_branch(
                            &mut values.riscv_code,
                            divisor_zero_branch,
                            divisor_zero_jump,
                            error_start,
                            encode::encode_bne,
                            JUNKREG0,
                            0,
                        );

                        /*
                         * Patch quotient overflow.
                         */
                        patch_far_branch(
                            &mut values.riscv_code,
                            overflow_branch,
                            overflow_jump,
                            error_start,
                            encode::encode_beq,
                            JUNKREG0,
                            0,
                        );

                        /*
                         * Patch fast/slow path selection.
                         */
                        let slow_offset =
                            ((slow_start as isize
                                - slow_branch as isize)
                                * 4)
                                as i32;

                        values.riscv_code[slow_branch] =
                            encode::encode_bne(
                                rdx,
                                0,
                                slow_offset,
                            );

                        /*
                         * Fast path -> normal end.
                         */
                        patch_far_jump(
                            &mut values.riscv_code,
                            fast_end_jump,
                            normal_end,
                        );

                        /*
                         * Slow path -> normal end.
                         */
                        patch_far_jump(
                            &mut values.riscv_code,
                            end_jump,
                            after_error,
                        );
                    } else {
                        /*
                         * 32-bit DIV:
                         *
                         * divisor = zero-extended source
                         */
                        values
                            .riscv_code
                            .push(encode::encode_slli(
                                JUNKREG1,
                                source_reg,
                                32,
                            ));

                        values
                            .riscv_code
                            .push(encode::encode_srli(
                                JUNKREG1,
                                JUNKREG1,
                                32,
                            ));

                        /*
                         * divisor != 0
                         */
                        values
                            .riscv_code
                            .push(encode::encode_sltu(
                                JUNKREG2,
                                JUNKREG1,
                                1,
                            ));

                        let divisor_zero_branch =
                            values.riscv_code.len();

                        values
                            .riscv_code
                            .push(encode::encode_bne(
                                JUNKREG2,
                                0,
                                0,
                            ));

                        let divisor_zero_jump =
                            emit_far_jump_placeholder(
                                &mut values.riscv_code,
                            );

                        /*
                         * EDX:EAX overflow check.
                         *
                         * JUNKREG0 = low 32 bits of EDX
                         */
                        values
                            .riscv_code
                            .push(encode::encode_slli(
                                JUNKREG0,
                                rdx,
                                32,
                            ));

                        values
                            .riscv_code
                            .push(encode::encode_srli(
                                JUNKREG0,
                                JUNKREG0,
                                32,
                            ));

                        /*
                         * If EDX >= divisor,
                         * unsigned DIV overflows.
                         */
                        values
                            .riscv_code
                            .push(encode::encode_sltu(
                                JUNKREG2,
                                JUNKREG0,
                                JUNKREG1,
                            ));

                        let overflow_branch =
                            values.riscv_code.len();

                        values
                            .riscv_code
                            .push(encode::encode_beq(
                                JUNKREG2,
                                0,
                                0,
                            ));

                        let overflow_jump =
                            emit_far_jump_placeholder(
                                &mut values.riscv_code,
                            );

                        /*
                         * Build:
                         *
                         *     JUNKREG0 = EDX:EAX
                         *
                         * as zero-extended 64-bit value.
                         */
                        values
                            .riscv_code
                            .push(encode::encode_slli(
                                JUNKREG0,
                                rax,
                                32,
                            ));

                        values
                            .riscv_code
                            .push(encode::encode_srli(
                                JUNKREG0,
                                JUNKREG0,
                                32,
                            ));

                        values
                            .riscv_code
                            .push(encode::encode_slli(
                                JUNKREG2,
                                rdx,
                                32,
                            ));

                        values
                            .riscv_code
                            .push(encode::encode_or(
                                JUNKREG0,
                                JUNKREG0,
                                JUNKREG2,
                            ));

                        /*
                         * DIVU/REMU on EDX:EAX.
                         */
                        values
                            .riscv_code
                            .push(encode::encode_remu(
                                JUNKREG2,
                                JUNKREG0,
                                JUNKREG1,
                            ));

                        values
                            .riscv_code
                            .push(encode::encode_divu(
                                rax,
                                JUNKREG0,
                                JUNKREG1,
                            ));

                        values
                            .riscv_code
                            .push(encode::encode_addi(
                                rdx,
                                JUNKREG2,
                                0,
                            ));

                        /*
                         * x86 32-bit write zero-extends
                         * EAX -> RAX and EDX -> RDX.
                         */
                        values
                            .riscv_code
                            .push(encode::encode_slli(
                                rax,
                                rax,
                                32,
                            ));

                        values
                            .riscv_code
                            .push(encode::encode_srli(
                                rax,
                                rax,
                                32,
                            ));

                        values
                            .riscv_code
                            .push(encode::encode_slli(
                                rdx,
                                rdx,
                                32,
                            ));

                        values
                            .riscv_code
                            .push(encode::encode_srli(
                                rdx,
                                rdx,
                                32,
                            ));

                        let end_jump =
                            emit_far_jump_placeholder(
                                &mut values.riscv_code,
                            );

                        let error_start =
                            values.riscv_code.len();

                        emit_divide_error(
                            &mut values,
                        );

                        let after_error =
                            values.riscv_code.len();

                        /*
                         * Patch divisor == 0.
                         */
                        patch_far_branch(
                            &mut values.riscv_code,
                            divisor_zero_branch,
                            divisor_zero_jump,
                            error_start,
                            encode::encode_bne,
                            JUNKREG2,
                            0,
                        );

                        /*
                         * Patch overflow.
                         */
                        patch_far_branch(
                            &mut values.riscv_code,
                            overflow_branch,
                            overflow_jump,
                            error_start,
                            encode::encode_beq,
                            JUNKREG2,
                            0,
                        );

                        /*
                         * Normal path -> after error block.
                         */
                        patch_far_jump(
                            &mut values.riscv_code,
                            end_jump,
                            after_error,
                        );
                    }
                }

                _ => todo!(),
            }
        }

        _ => unreachable!(),
    }

    HandelerReturnValue::new(values.operation.len)
}