#!/bin/zsh

## bootloader

BOOT_LOADER_DIR=$(dirname "$0")/bootloader
cd $BOOT_LOADER_DIR
cargo build --target x86_64-unknown-uefi
