.PHONY: all build copy clean

all: build copy
	:

clean:
	rm -f target/armv7-unknown-linux-musleabihf/release/kindle-events-screen

target/armv7-unknown-linux-musleabihf/release/kindle-events-screen: src/*rs
	SYSROOT_LIB_DIR=~/git/Amazon-Kindle-Cross-Toolchain/arm-kindle-linux-gnueabi/arm-kindle-linux-gnueabi/sysroot/lib/ \
	cargo build --target armv7-unknown-linux-musleabihf --release

build: target/armv7-unknown-linux-musleabihf/release/kindle-events-screen
	arm-linux-gnueabi-strip target/armv7-unknown-linux-musleabihf/release/kindle-events-screen

copy: target/armv7-unknown-linux-musleabihf/release/kindle-events-screen
	scp target/armv7-unknown-linux-musleabihf/release/kindle-events-screen root@192.168.2.106:/var/tmp/root
