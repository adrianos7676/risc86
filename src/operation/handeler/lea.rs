use crate::{
    X86Reg,
    operation::handeler::{HandelerInputValue, HandelerReturnValue},
};

pub fn lea(values: HandelerInputValue) -> HandelerReturnValue {
    if values.code[values.code_offset] != 0x48 || values.code[values.code_offset + 1] != 0x8D {
        todo!();
    }

    let modrm = values.code[values.code_offset + 2];
    let x86_register = X86Reg::from_modrm_reg(modrm, false);

    let mode = modrm >> 6;
    let rm = modrm & 0b111;

    if mode != 0b00 || rm != 0b101 {
        todo!();
    }

    let disp = i32::from_le_bytes(values.code[values.code_offset + 3..values.code_offset + 7].try_into().unwrap());

    let next_rip = values.header.e_entry + (values.code_offset + values.operation.len) as u64;

    let address = (next_rip as i64 + disp as i64) as u64;

    let target_segment = values.segments
        .iter()
        .find(|segment| segment.p_vaddr <= address && address < segment.p_vaddr + segment.p_filesz)
        .unwrap();

    let target_file_offset = target_segment.p_offset + (address - target_segment.p_vaddr);

    let data_start = target_file_offset as usize;
    let data_end = (target_segment.p_offset + target_segment.p_filesz) as usize;

    let data = &values.bytes[data_start..data_end];

    dbg!(x86_register);
    dbg!(address);
    dbg!(target_file_offset);
    dbg!(data);

    values.riscv_data.extend_from_slice(data);

    let fixup_index = values.riscv_code.len();

    values.lea_fixups.push((fixup_index, x86_register.to_riscv(), address));

    values.riscv_code.push(0);
    values.riscv_code.push(0);

    HandelerReturnValue { operation_len: values.operation.len }
}
