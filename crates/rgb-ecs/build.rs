fn main() {
    // Required for tango-bench dynamic linking
    #[cfg(not(windows))]
    println!("cargo:rustc-link-arg-benches=-rdynamic");

    println!("cargo:rerun-if-changed=build.rs");
}
