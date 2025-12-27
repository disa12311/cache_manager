fn main() {
    // Enable mobile cfg for Tauri
    println!("cargo:rustc-check-cfg=cfg(mobile)");
    
    tauri_build::build()
}