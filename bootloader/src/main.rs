#![no_main]
#![no_std]
#![macro_use]
extern crate alloc;

use log::info;
use uefi::prelude::*;
use uefi::proto::media::file::{File, FileAttribute, FileMode, FileType};
use uefi::CString16;
use uefi::Result;

#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    info!("Hello world!");
    let boot_services = system_table.boot_services();
    match save_memory_map(boot_services) {
        Ok(_) => (),
        Err(_) => {
            info!("save_memory_map failed");
            return Status::ABORTED;
        }
    }
    system_table.boot_services().stall(10_000_000);
    Status::SUCCESS
}

fn save_memory_map(boot_services: &BootServices) -> Result {
    // open file
    let mut fs_protocol = boot_services
        .get_image_file_system(boot_services.image_handle())
        .unwrap();

    let mut root = fs_protocol.open_volume().unwrap();

    let memmap_file_name = CString16::try_from("memmap").unwrap();
    let memmap_file_handle = root
        .open(
            &memmap_file_name,
            FileMode::CreateReadWrite,
            FileAttribute::empty(),
        )
        .unwrap();
    let mut memmap_file = match memmap_file_handle.into_type().unwrap() {
        FileType::Regular(file) => file,
        FileType::Dir(_) => {
            panic!();
        }
    };

    //write memory map to file
    memmap_file
        .write("Index, Type, Type(name), PhysicalStart, NumberOfPages, Attribute\n".as_bytes())
        .unwrap();

    let mut memmap_buf = alloc::vec![0; 4096 * 4];
    let memmap = boot_services.memory_map(&mut memmap_buf).unwrap();

    for (i, descriptor) in memmap.entries().enumerate() {
        memmap_file
            .write(
                alloc::format!(
                    "{}, {:x}, {:?}, {:08x}, {:x}, {:x}\n",
                    i,
                    descriptor.ty.0,
                    descriptor.ty,
                    descriptor.phys_start,
                    descriptor.page_count,
                    descriptor.att,
                )
                .as_bytes(),
            )
            .unwrap();
    }
    memmap_file.close();

    Ok(())
}
