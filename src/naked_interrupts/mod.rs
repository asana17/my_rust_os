mod idt;
use bitflags::bitflags;
use crate::println;
use core::arch::asm;
use lazy_static::lazy_static;

macro_rules! handler {
    ($name:ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!("mov rdi, rsp",
                     "sub rsp, 8", //align the stack pointer
                    concat!("call ", stringify!($name)), // call handler_function
                    options(noreturn),
                )
            }
        }
        wrapper
    }}
}

macro_rules! handler_with_error_code {
    ($name:ident) => {{
        #[naked]
        extern "C" fn wrapper() -> ! {
            unsafe {
                asm!("pop rsi", //pop error code into rsi
                     "mov rdi, rsp",
                     "sub rsp, 8",
                     concat!("call ", stringify!($name)),
                     options(noreturn),
                )
            }
        }
        wrapper
    }}
}


lazy_static!{
    static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();
        //idt.set_handler(0, divide_by_zero_wrapper);
        idt.set_handler(0, handler!(divide_by_zero_handler));
        idt.set_handler(6, handler!(invalid_opcode_handler));
        idt.set_handler(14, handler_with_error_code!(page_fault_handler));
        idt
    };
}

#[no_mangle]
extern "C" fn divide_by_zero_handler(stack_frame: &ExceptionStackFrame) -> ! {

    println!("\nEXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);

    loop {}
}

#[no_mangle]
extern "C" fn invalid_opcode_handler(stack_frame: &ExceptionStackFrame) -> ! {
    println!("\nEXCEPTION: INVALID OPCODE at {:#x}\n{:#?}", stack_frame.instruction_pointer, stack_frame);

    loop {}
}

#[no_mangle]
extern "C" fn page_fault_handler(stack_frame: &ExceptionStackFrame, error_code:u64) -> ! {
    use x86_64::registers::control;
    println!("\nEXCEPTION: PAGE FAULT while accessing {:#x}\nerror code: {:?}\n{:#?}", control::Cr2::read(), PageFaultErrorCode::from_bits(error_code).unwrap(), stack_frame);

    loop {};
}

bitflags! {
    struct PageFaultErrorCode : u64 {
        const PROTECTION_VIOLATION = 1 << 0;
        const CAUSED_BY_WRITE = 1 << 1;
        const USER_MODE = 1 << 2;
        const MALFORMED_TABLE = 1 << 3;
        const INSTRUCTION_FETCH = 1 << 4;
    }
}


pub fn init() {
    IDT.load();
}

#[derive(Debug)]
#[repr(C)]
struct ExceptionStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}
