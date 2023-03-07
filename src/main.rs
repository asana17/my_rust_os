#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(naked_functions)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use blog_os::println;
//use core::arch::asm;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    blog_os::init();

    //x86_64::instructions::interrupts::int3();
    //blog_os::divide_by_zero();
    //unsafe {asm!("ud2")};
    unsafe {*(0xdeadbeaf as *mut u64) = 42};

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}
