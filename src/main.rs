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

    use x86_64::registers::control::Cr3;

    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at : {:?}", level_4_page_table.start_address());
    //blog_os::divide_by_zero();
    //unsafe {asm!("ud2")};
    /*let ptr = 0x2031b2 as *mut u32;
    unsafe { let x = *ptr; }
    println!("read worked");
    unsafe { *ptr = 42; }*/
    println!("write worked");
    //unsafe {*(0xdeadbeaf as *mut u64) = 42};

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    blog_os::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    blog_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    blog_os::hlt_loop();
}
