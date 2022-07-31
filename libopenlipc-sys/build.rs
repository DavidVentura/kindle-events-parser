use std::env;
fn main() {
    env::set_var("SYSROOT_LIB_DIR", "Amazon-Kindle-Cross-Toolchain/arm-kindle-linux-gnueabi/arm-kindle-linux-gnueabi/sysroot/lib/");

    println!("cargo:rustc-link-search=so");
    //println!("cargo:rustc-link-search={}", sysroot_lib_dir);
    println!("cargo:rustc-link-lib=glib-2.0");
    println!("cargo:rustc-link-lib=dbus-1");
    println!("cargo:rustc-link-lib=gthread-2.0");
}
