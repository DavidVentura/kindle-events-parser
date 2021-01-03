fn main() {
    println!("cargo:rustc-link-search=so");
    println!("cargo:rustc-link-search=/home/david/git/Amazon-Kindle-Cross-Toolchain/arm-kindle-linux-gnueabi/arm-kindle-linux-gnueabi/sysroot/lib/");
    println!("cargo:rustc-link-lib=glib-2.0");
    println!("cargo:rustc-link-lib=dbus-1");
    println!("cargo:rustc-link-lib=gthread-2.0");
}
