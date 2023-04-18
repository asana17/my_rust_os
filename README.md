## my_rust_os
This OS is based on [blog_os](https://os.phil-opp.com) and [mikan_os](https://github.com/uchan-nos/mikanos).

## objdump

```
objdump -m i386:x86-64 -b binary -D target/x86_64-blog_os/debug/bootimage-blog_os.b
in > objdump.txt
```
start address offset: `0x1f2e00`
ex. if exception at `0x203ef7` in QEMU, look at `110f7` in objdump result
