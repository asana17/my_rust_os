#![no_main]
#![no_std]
#![macro_use]
extern crate alloc;

use core::cmp;
use elf_rs::*;
use log::info;
use uefi::data_types::Align;
use uefi::prelude::*;
use uefi::proto::media::file::{Directory, File, FileAttribute, FileInfo, FileMode, FileType};
use uefi::table::boot::{AllocateType, MemoryType};
use uefi::CString16;
use uefi::Result;

#[entry]
fn main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();
    info!("Hello world!");
    let kernel_start_addr;
    {
        let boot_services = system_table.boot_services();
        let mut fs_protocol = boot_services
            .get_image_file_system(boot_services.image_handle())
            .unwrap();

        let mut root = fs_protocol.open_volume().unwrap();

        // save memory map
        match save_memory_map(&boot_services, &mut root) {
            Ok(_) => (),
            Err(_) => {
                info!("failed to save memory map");
                return Status::ABORTED;
            }
        }

        // load kernel elf
        kernel_start_addr = match load_kernel_elf(&boot_services, &mut root) {
            Ok(addr) => addr,
            Err(_) => {
                info!("failed to load kernel elf");
                return Status::ABORTED;
            }
        };
    }
    // exit uefi boot service
    let _ = system_table.exit_boot_services();
    let entry_point =
        unsafe { core::mem::transmute::<u64, extern "sysv64" fn() -> ()>(kernel_start_addr) };

    entry_point();

    Status::SUCCESS
}

fn save_memory_map(boot_services: &BootServices, root: &mut Directory) -> Result {
    // open file
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
            panic!("failed to open memmap file: is directory");
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

fn load_kernel_elf(boot_services: &BootServices, root: &mut Directory) -> Result<u64> {
    // read kernel elf file from root directory
    let kernel_elf_name = CString16::try_from("kernel.elf").unwrap();
    let kernel_elf_handle = root
        .open(&kernel_elf_name, FileMode::Read, FileAttribute::READ_ONLY)
        .unwrap();
    let mut kernel_elf = match kernel_elf_handle.into_type().unwrap() {
        FileType::Regular(file) => file,
        FileType::Dir(_) => {
            panic!("failed to open kernel file: is directory");
        }
    };

    // get kernel elf size from file info
    const KERNEL_INFO_SIZE: usize = 400;
    let mut buf = [0u8; KERNEL_INFO_SIZE];

    assert!((buf.as_ptr() as usize) % <FileInfo as Align>::alignment() == 0);

    let kernel_info: &FileInfo = kernel_elf.get_info(&mut buf).unwrap();
    let kernel_elf_size = kernel_info.file_size() as usize;

    // allocate temporal buffer for kernel file
    let raw_kernel_elf_buffer = boot_services
        .allocate_pool(MemoryType::LOADER_DATA, kernel_elf_size)
        .unwrap();
    let kernel_elf_buffer =
        unsafe { core::slice::from_raw_parts_mut(raw_kernel_elf_buffer, kernel_elf_size) };
    let read_size = kernel_elf.read(kernel_elf_buffer).unwrap();
    kernel_elf.close();

    assert_eq!(kernel_elf_size, read_size);

    // calc address range of LOAD segments from elf program header
    let elf = Elf::from_bytes(kernel_elf_buffer).unwrap();
    let elf64 = match elf {
        Elf::Elf64(elf) => elf,
        _ => panic!("got Elf32, expected Elf64"),
    };

    let (start_addr, end_addr) = calc_load_address_range(&elf64);

    // allocate memory for LOAD segments
    let num_pages = ((end_addr - start_addr + 0xfff) / 0x1000) as usize;
    let allocated_addr = boot_services
        .allocate_pages(
            AllocateType::Address(start_addr),
            MemoryType::LOADER_DATA,
            num_pages,
        )
        .unwrap();

    // copy LOAD segments to allocated memory
    copy_load_segments(boot_services, &elf64);
    info!("kernel: {:#x} - {:#x}\n", start_addr, end_addr);
    boot_services.free_pool(raw_kernel_elf_buffer).unwrap();

    Ok(allocated_addr)
}

fn calc_load_address_range(elf64: &Elf64) -> (u64, u64) {
    let phdr_iter = elf64.program_header_iter();
    let mut start_addr = u64::max_value();
    let mut end_addr = 0;

    for phdr in phdr_iter {
        if phdr.ph_type() != ProgramType::LOAD {
            continue;
        }
        start_addr = cmp::min(start_addr, phdr.vaddr());
        end_addr = cmp::max(end_addr, phdr.vaddr() + phdr.memsz());
    }

    return (start_addr, end_addr);
}

fn copy_load_segments(boot_services: &BootServices, elf64: &Elf64) {
    let phdr_iter = elf64.program_header_iter();

    for phdr in phdr_iter {
        if phdr.ph_type() != ProgramType::LOAD {
            continue;
        }
        let load_segment_addr = phdr.paddr() as *mut u8;
        let dest_addr = phdr.vaddr() as *mut u8;
        unsafe { boot_services.memmove(dest_addr, load_segment_addr, phdr.filesz() as usize) };
        let remain_bytes = (phdr.memsz() - phdr.filesz()) as usize;
        let remain_addr = (phdr.vaddr() + phdr.filesz()) as *mut u8;
        unsafe { boot_services.set_mem(remain_addr, remain_bytes, 0) }
    }
}
