use crate::{X86Reg, elf::{self, Elf64Header}};

pub fn lea(
    modrm: u8,
    code: &[u8],
    code_offset: usize,
    header: &Elf64Header,
    segments: &[elf::Elf64ProgramHeader],
    riscv_code: &mut Vec<u32>,
    riscv_data: &mut Vec<u8>,
    bytes: &[u8],
    lea_fixup: &mut Option<(usize, u8)>,
) -> usize {
    let x86_register = X86Reg::from_modrm_reg(modrm);

    let disp = i32::from_le_bytes(
        code[code_offset + 3..code_offset + 7]
            .try_into()
            .unwrap(),
    );

    let next_rip = header.e_entry + (code_offset + 7) as u64;

    let address = (next_rip as i64 + disp as i64) as u64;

    let target_segment = segments
        .iter()
        .find(|segment| {
            segment.p_vaddr <= address
                && address < segment.p_vaddr + segment.p_filesz
        })
        .unwrap();

    let target_file_offset =
        target_segment.p_offset + (address - target_segment.p_vaddr);

    let data_start = target_file_offset as usize;
    let data_end =
        (target_segment.p_offset + target_segment.p_filesz) as usize;

    let data = &bytes[data_start..data_end];

    riscv_data.extend_from_slice(data);

    *lea_fixup = Some((
        riscv_code.len(),
        x86_register.to_riscv(),
    ));

    riscv_code.push(0);
    riscv_code.push(0);

    7
}