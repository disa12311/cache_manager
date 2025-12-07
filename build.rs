// build.rs - Script build để cấu hình Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Chỉ áp dụng cho Windows
    #[cfg(target_os = "windows")]
    {
        // Đặt subsystem thành Windows để ẩn console
        if std::env::var("PROFILE").unwrap_or_default() == "release" {
            println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
            println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");
        }
    }
    
    // Thông báo khi build
    println!("cargo:rerun-if-changed=build.rs");
}