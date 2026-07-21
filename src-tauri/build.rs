fn main() {
    tauri_build::build();

    #[cfg(windows)]
    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let resource = std::path::PathBuf::from(out_dir).join("resource.lib");
        println!("cargo:rustc-link-arg-tests={}", resource.display());
    }
}
