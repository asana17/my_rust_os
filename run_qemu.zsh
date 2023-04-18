#!/usr/bin/zsh -ex

if [ $# -lt 1 ]
then
    echo "Usage: $0 <.efi file> [another file]"
    exit 1
fi


WORKSPACE_DIR=$(dirname "$0")
EFI_FILE=$1
ANOTHER_FILE=$2
DISK_IMG=./disk.img
MOUNT_POINT=./mnt

$WORKSPACE_DIR/build.zsh
$WORKSPACE_DIR/make_image.zsh $DISK_IMG $MOUNT_POINT $EFI_FILE $ANOTHER_FILE
$WORKSPACE_DIR/run_image.zsh $DISK_IMG
