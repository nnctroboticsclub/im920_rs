fn main() {
    println!("cargo:rerun-if-changed=.");

    cbindgen::Builder::new()
        .with_crate(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .with_language(cbindgen::Language::Cxx)
        .with_include("srobo2/ffi/common.hpp")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("inc/srobo2/ffi/im920.hpp");
}
