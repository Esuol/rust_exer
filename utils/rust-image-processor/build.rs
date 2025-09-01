extern crate napi_build;

fn main() {
    napi_build::setup();

    println!("cargo:rustc-link-lib=image-processor");
    println!("cargo:rustc-link-lib=image-processor-sys");
    println!("cargo:rustc-link-lib=image-processor-sys-sys");
    println!("cargo:rustc-link-lib=image-processor-sys-sys-sys");
    println!("cargo:rustc-link-lib=image-processor-sys-sys-sys-sys");
    println!("cargo:rustc-link-lib=image-processor-sys-sys-sys-sys-sys");
    println!("cargo:rustc-link-lib=image-processor-sys-sys-sys-sys-sys-sys");
}
