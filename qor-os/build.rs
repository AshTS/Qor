use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let core_count: usize = env::var_os("CORE_COUNT").map(|v| v.to_string_lossy().to_string()).unwrap_or("2".into()).parse().unwrap();
    let dest_path = Path::new(&out_dir).join("core_count.rs");
    fs::write(
        dest_path,
        format!("pub const CORE_COUNT: usize = {core_count};")
    ).unwrap();
    println!("cargo:rerun-if-env-changed=CORE_COUNT");
}