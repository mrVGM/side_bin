fn main() {
    println!("cargo::rerun-if-changed=lib");
    println!("cargo:rustc-link-search=native=lib");
    println!("cargo:rustc-link-lib=static=fs_monitor");
    tauri_build::build()
}
