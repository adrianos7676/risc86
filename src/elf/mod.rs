use std::fs;


impl Elf64Header {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            e_ident: bytes[0..16].try_into().unwrap(),
            e_type: u16::from_le_bytes(bytes[16..18].try_into().unwrap()),
            e_machine: u16::from_le_bytes(bytes[18..20].try_into().unwrap()),
            e_version: u32::from_le_bytes(bytes[20..24].try_into().unwrap()),
            e_entry: u64::from_le_bytes(bytes[24..32].try_into().unwrap()),
            e_phoff: u64::from_le_bytes(bytes[32..40].try_into().unwrap()),
            e_shoff: u64::from_le_bytes(bytes[40..48].try_into().unwrap()),
            e_flags: u32::from_le_bytes(bytes[48..52].try_into().unwrap()),
            e_ehsize: u16::from_le_bytes(bytes[52..54].try_into().unwrap()),
            e_phentsize: u16::from_le_bytes(bytes[54..56].try_into().unwrap()),
            e_phnum: u16::from_le_bytes(bytes[56..58].try_into().unwrap()),
            e_shentsize: u16::from_le_bytes(bytes[58..60].try_into().unwrap()),
            e_shnum: u16::from_le_bytes(bytes[60..62].try_into().unwrap()),
            e_shstrndx: u16::from_le_bytes(bytes[62..64].try_into().unwrap()),
        }
    }
}

impl Elf64ProgramHeader {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            p_type: u32::from_le_bytes(bytes[0..4].try_into().unwrap()),
            p_flags: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            p_offset: u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
            p_vaddr: u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
            p_paddr: u64::from_le_bytes(bytes[24..32].try_into().unwrap()),
            p_filesz: u64::from_le_bytes(bytes[32..40].try_into().unwrap()),
            p_memsz: u64::from_le_bytes(bytes[40..48].try_into().unwrap()),
            p_align: u64::from_le_bytes(bytes[48..56].try_into().unwrap()),
        }
    }
}

#[derive(Debug)]
pub struct Elf64Header {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct Elf64ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

pub fn write_elf(output_name: String, source_risc_bytes: Vec<u8>, source_risc_data: Vec<u8>) {
    let code_offset = 0x1000u64;
    let code_vaddr = 0x10000u64;

    let data_offset = 0x2000u64;
    let data_vaddr = 0x11000u64;

    let entry = 0x10000u64;
    let code_size = source_risc_bytes.len() as u64;
    let data_size = source_risc_data.len() as u64;

    let header = Elf64Header {
        e_ident: [
            0x7f, b'E', b'L', b'F',
            2,
            1,
            1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        e_type: 2,
        e_machine: 243,
        e_version: 1,
        e_entry: entry,
        e_phoff: 64,
        e_shoff: 0,
        e_flags: 0,
        e_ehsize: 64,
        e_phentsize: 56,
        e_phnum: 2,
        e_shentsize: 0,
        e_shnum: 0,
        e_shstrndx: 0,
    };

    let code_segment = Elf64ProgramHeader {
        p_type: 1,
        p_flags: 5,
        p_offset: code_offset,
        p_vaddr: code_vaddr,
        p_paddr: code_vaddr,
        p_filesz: code_size,
        p_memsz: code_size,
        p_align: 0x1000,
    };

    let data_segment = Elf64ProgramHeader {
        p_type: 1,
        p_flags: 4,
        p_offset: data_offset,
        p_vaddr: data_vaddr,
        p_paddr: data_vaddr,
        p_filesz: data_size,
        p_memsz: data_size,
        p_align: 0x1000,
    };

    let mut bytes = Vec::new();

    bytes.extend_from_slice(&header.e_ident);
    bytes.extend_from_slice(&header.e_type.to_le_bytes());
    bytes.extend_from_slice(&header.e_machine.to_le_bytes());
    bytes.extend_from_slice(&header.e_version.to_le_bytes());
    bytes.extend_from_slice(&header.e_entry.to_le_bytes());
    bytes.extend_from_slice(&header.e_phoff.to_le_bytes());
    bytes.extend_from_slice(&header.e_shoff.to_le_bytes());
    bytes.extend_from_slice(&header.e_flags.to_le_bytes());
    bytes.extend_from_slice(&header.e_ehsize.to_le_bytes());
    bytes.extend_from_slice(&header.e_phentsize.to_le_bytes());
    bytes.extend_from_slice(&header.e_phnum.to_le_bytes());
    bytes.extend_from_slice(&header.e_shentsize.to_le_bytes());
    bytes.extend_from_slice(&header.e_shnum.to_le_bytes());
    bytes.extend_from_slice(&header.e_shstrndx.to_le_bytes());

    for segment in [code_segment, data_segment] {
        bytes.extend_from_slice(&segment.p_type.to_le_bytes());
        bytes.extend_from_slice(&segment.p_flags.to_le_bytes());
        bytes.extend_from_slice(&segment.p_offset.to_le_bytes());
        bytes.extend_from_slice(&segment.p_vaddr.to_le_bytes());
        bytes.extend_from_slice(&segment.p_paddr.to_le_bytes());
        bytes.extend_from_slice(&segment.p_filesz.to_le_bytes());
        bytes.extend_from_slice(&segment.p_memsz.to_le_bytes());
        bytes.extend_from_slice(&segment.p_align.to_le_bytes());
    }

    bytes.resize(code_offset as usize, 0);
    bytes.extend_from_slice(&source_risc_bytes);

    bytes.resize(data_offset as usize, 0);
    bytes.extend_from_slice(&source_risc_data);

    fs::write(output_name, bytes).unwrap();
}

