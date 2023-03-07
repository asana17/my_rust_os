mod idt;
use crate::println;
use core::arch::asm;
use lazy_static::lazy_static;

lazy_static!{
    static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();
        idt.set_handler(0, divide_by_zero_wrapper);
        idt
    };
}

#[naked]
extern "C" fn divide_by_zero_wrapper() -> ! {
    unsafe {
        asm!("mov rdi, rsp", "sub rsp, 8",  "call divide_by_zero_handler", options(noreturn));
    }
}

#[no_mangle]
extern "C" fn divide_by_zero_handler(stack_frame: &ExceptionStackFrame) -> ! {

    println!("\nEXCEPTION: DIVIDE BY ZERO{:#?}\n", &*stack_frame);

    loop {}
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
