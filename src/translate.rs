use crate::operation::{self, handeler::TranslationContext};

#[derive(Debug)]
pub struct TranslationResult {
    pub code: Vec<u32>,
    pub data: Vec<u8>,
    pub lea_fixups: Vec<(usize, u8, u64)>,
    pub branch_fixups: Vec<(usize, usize)>,
    pub call_fixups: Vec<(usize, usize)>,
    pub code_address: usize,
    pub children: Vec<TranslationResult>,
}

pub async fn translate(
    translation_context: TranslationContext,
    code_offset: usize,
    mut registers: [u64; 16],
) -> TranslationResult {
    let code_address = code_offset;

    let mut riscv_code = Vec::new();
    let mut riscv_data = Vec::new();
    let mut lea_fixups = Vec::new();
    let mut branch_fixups = Vec::new();
    let mut call_fixups = Vec::new();
    let mut code_offset = code_offset;
    let mut children = Vec::new();

    while code_offset < translation_context.code.len() {
        dbg!(code_offset);
        dbg!(translation_context.code[code_offset]);

        let operation =
            operation::X86operation::decode(
                &translation_context.code,
                code_offset,
            );

        dbg!(&operation);

        let handeler_input_value =
            operation::handeler::HandelerInputValue {
                code: &translation_context.code,
                code_offset,
                riscv_code: &mut riscv_code,
                registers: &mut registers,
                operation: &operation,
                header: &translation_context.header,
                bytes: &translation_context.bytes,
                riscv_data: &mut riscv_data,
                lea_fixups: &mut lea_fixups,
                branch_fixups: &mut branch_fixups,
                call_fixups: &mut call_fixups,
                segments: &translation_context.segments,
                translation_context: &translation_context,
            };

        let handler_return_value = match operation.operation {
            operation::X86operation::Syscall => {
                operation::handeler::syscall::syscall(
                    handeler_input_value,
                )
            }
            operation::X86operation::Mov => {
                operation::handeler::mov::mov(
                    handeler_input_value,
                )
            }
            operation::X86operation::Lea => {
                operation::handeler::lea::lea(
                    handeler_input_value,
                )
            }
            operation::X86operation::Xor => {
                operation::handeler::xor::xor(
                    handeler_input_value,
                )
            }
            operation::X86operation::Endbr64 => {
                operation::handeler::endbr64::endbr64(
                    handeler_input_value,
                )
            }
            operation::X86operation::ConditionalJump => {
                operation::handeler::conditionaljump::conditionaljump(
                    handeler_input_value,
                )
            }
            operation::X86operation::Add => {
                operation::handeler::add::add(
                    handeler_input_value,
                )
            }
            operation::X86operation::Sub => {
                operation::handeler::sub::sub(
                    handeler_input_value,
                )
            }
            operation::X86operation::And => {
                operation::handeler::and::and(
                    handeler_input_value,
                )
            }
            operation::X86operation::Or => {
                operation::handeler::or::or(
                    handeler_input_value,
                )
            }
            operation::X86operation::Cmp => {
                operation::handeler::cmp::cmp(
                    handeler_input_value,
                )
            }
            operation::X86operation::Push => {
                operation::handeler::push::push(
                    handeler_input_value,
                )
            }
            operation::X86operation::Pop => {
                operation::handeler::pop::pop(
                    handeler_input_value,
                )
            }
            operation::X86operation::Nop => {
                operation::handeler::nop::nop(
                    handeler_input_value,
                )
            }

            operation::X86operation::Test => {
                operation::handeler::test::test(
                    handeler_input_value,
                )
            }

            operation::X86operation::Jmp => {
                operation::handeler::jmp::jmp(
                    handeler_input_value,
                )
            }
            operation::X86operation::Call => {
                operation::handeler::call::call(
                    handeler_input_value,
                )
            }

            operation::X86operation::Ret => {
                operation::handeler::ret::ret(
                    handeler_input_value,
                )
            },
        };

        dbg!(code_offset);
dbg!(handler_return_value.operation_len);
dbg!(handler_return_value.finish_thread);
dbg!(handler_return_value.future.is_some());
        code_offset += handler_return_value.operation_len;

        if let Some(child) = handler_return_value.future {
            children.push(child);
        }

        if handler_return_value.finish_thread {
            break;
        }
    }

    let mut child_results = Vec::new();

    for child in children {
        match child.await {
            Ok(result) => {
                child_results.push(result);
            }
            Err(error) => {
                dbg!(error);
            }
        }
    }

    dbg!(code_address);
    dbg!(&child_results);

    TranslationResult {
        code: riscv_code,
        data: riscv_data,
        lea_fixups,
        branch_fixups,
        call_fixups,
        code_address,
        children: child_results,
    }
}