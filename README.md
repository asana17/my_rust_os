## my_rust_os
This OS is based on [blog_os](https://os.phil-opp.com).

## objdump

```
objdump -m i386:x86-64 -b binary -D target/x86_64-blog_os/debug/bootimage-blog_os.b
in > objdump.txt
```
offset: 0x1f2e00
if exception at 0x203ef7
look at 110f7
