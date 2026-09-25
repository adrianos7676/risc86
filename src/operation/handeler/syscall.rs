use crate::{SysCall, encode, operation::handeler::{HandelerInputValue, HandelerReturnValue}};

pub fn syscall(values: HandelerInputValue) -> HandelerReturnValue {
    let syscall_number = values.registers[0];

    let syscall = SysCall::from_x86(syscall_number);
    let riscv_syscall = syscall.to_riscv();

    let mut finish_thread = false;

    match syscall {
        SysCall::Write => {
            values.riscv_code.push(encode::encode_addi(5, 13, 0));
            values.riscv_code.push(encode::encode_addi(6, 12, 0));
            values.riscv_code.push(encode::encode_addi(12, 11, 0));

            values.riscv_code.push(encode::encode_addi(11, 6, 0));
            values.riscv_code.push(encode::encode_addi(10, 5, 0));
        }

        SysCall::Exit => {
            values.riscv_code.push(encode::encode_addi(10, 13, 0));
            finish_thread = true;
        }
    }

    values.riscv_code.push(encode::encode_addi(17, 0, riscv_syscall as i32));

    values.riscv_code.push(encode::encode_ecall());

    HandelerReturnValue::finish(values.operation.len)
}
