fn main() {
    println!("cargo:rustc-link-search=native=lib");
    println!("cargo:rustc-link-lib=static=fs_monitor");
    tauri_build::build()
}
