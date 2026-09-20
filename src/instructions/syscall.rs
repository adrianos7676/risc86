use crate::{SysCall, encode};


//return value is whats added to the Program Offset
pub fn syscall(
    registers: &[u64; 16],
    riscv_code: &mut Vec<u32>,
) -> usize {
    let syscall_number = registers[0];

    let syscall = SysCall::from_x86(syscall_number);

    let riscv_syscall = syscall.to_riscv();

    match syscall {
        SysCall::Write => {
            riscv_code.push(encode::encode_addi(5, 13, 0));
            riscv_code.push(encode::encode_addi(6, 12, 0));
            riscv_code.push(encode::encode_addi(12, 11, 0));

            riscv_code.push(encode::encode_addi(11, 6, 0));
            riscv_code.push(encode::encode_addi(10, 5, 0));
        }

        SysCall::Exit => {
            riscv_code.push(encode::encode_addi(10, 13, 0));
        }
    }

    riscv_code.push(encode::encode_addi(17, 0, riscv_syscall as i32));

    riscv_code.push(encode::encode_ecall());

    2
}
