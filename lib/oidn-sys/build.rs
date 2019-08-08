extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-link-lib=OpenImageDenoise");
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .whitelist_function("oidn.*")
        .whitelist_type("OIDN.*")
        .layout_tests(true)
        .generate_comments(true)
        .prepend_enum_name(false)
        .generate()
        .expect("bindgen failed to generate");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("bindgen failed to write bindings.rs");
}
