# Event parser for kindle

Output looks like:

```
[com.lab126.powerd] battLevelChanged 67
This is a <BatteryChanged> Event
Battery at 67%
```

I will likely hook up some function to the events later on.

# Build

You need to have an ARMv7 linker installed, you can do so with `sudo apt-get install gcc-9-arm-linux-gnueabihf`.

You have to add `rustup target add arm-unknown-linux-musleabihf`

```
[target.armv7-unknown-linux-musleabi]
linker = "arm-linux-gnueabihf-gcc-9"
```
