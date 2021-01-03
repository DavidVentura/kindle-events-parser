Get the [toolchain](https://github.com/samsheff/Amazon-Kindle-Cross-Toolchain/tree/master/arm-kindle-linux-gnueabi)
Configure it in your `.cargo/config`:

```
[target.armv7-unknown-linux-gnueabi]
linker = "Amazon-Kindle-Cross-Toolchain/arm-kindle-linux-gnueabi/bin/arm-kindle-linux-gnueabi-cc"
```


Uses OpenLIPC headers: https://github.com/Arkq/openlipc
Docs at https://arkq.github.io/openlipc

List of events
https://www.mobileread.com/forums/showthread.php?t=227859

Some docs
https://medium.com/dwelo-r-d/wrapping-unsafe-c-libraries-in-rust-d75aeb283c65



Added .so's from the kindle
