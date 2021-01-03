# LibOpenLIPC-sys

Kindle liblipc wrapper in rust. Only a very small subset of functionality is exposed.

Used [OpenLIPC headers](https://github.com/Arkq/openlipc) and converted them with `bindgen`. [Docs are here](https://arkq.github.io/openlipc).

Get the [toolchain](https://github.com/samsheff/Amazon-Kindle-Cross-Toolchain/tree/master/arm-kindle-linux-gnueabi)
Configure it in your `.cargo/config`:

```
[target.armv7-unknown-linux-gnueabi]
linker = "Amazon-Kindle-Cross-Toolchain/arm-kindle-linux-gnueabi/bin/arm-kindle-linux-gnueabi-cc"
```

Export the path to sysroot/lib before compiling:
```
export SYSROOT_LIB_DIR=~/git/Amazon-Kindle-Cross-Toolchain/arm-kindle-linux-gnueabi/arm-kindle-linux-gnueabi/sysroot/lib/
```

Copy the .so files from the repo to `SYSROOT_LIB_DIR` (these I got from my kindle):
```
cp -vt $SYSROOT_LIB_DIR so/*
```

## Useful links

* [List of LIPC events](https://www.mobileread.com/forums/showthread.php?t=227859)
* [Wrapping unsafe C libs in rust](https://medium.com/dwelo-r-d/wrapping-unsafe-c-libraries-in-rust-d75aeb283c65)

## Misc

I tried to run the binaries with `qemu-arm-static` but they segfault. No idea.

## Thanks

To `pie_flavor` in the rust discord channel who explained to me the whole FFI thing.
