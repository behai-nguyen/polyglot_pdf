// 26/10/2025

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // println!("cargo:rustc-link-lib=dylib=harfbuzz");
    // println!("cargo:rustc-link-lib=static=harfbuzz");
    println!("cargo:rustc-link-lib=harfbuzz");
    println!("cargo:rustc-link-lib=harfbuzz-subset");

    // C:\PF\harfbuzz\src\[*.h, hb.h]
    // /usr/local/include/harfbuzz/[*.h, hb.h]    
    // C:\PF\harfbuzz\build\src\harfbuzz.lib
    // /usr/local/lib/x86_64-linux-gnu/libharfbuzz.so.0
    let (hb_include, lib_search) = if cfg!(target_os = "windows") {
        (
            "C:/PF/harfbuzz/src/",
            "C:/PF/harfbuzz/build/src/",
        )        
    } else {
        (
            "/usr/local/include/harfbuzz/",
            "/usr/local/lib/x86_64-linux-gnu/",
        )
    };

    println!("cargo:rustc-link-search=native={}", lib_search);

    // Windows vs Linux include paths
    let mut clang_args = Vec::new();

    // Try to auto-detect GCC’s include path on Unix-like systems
    // Handle the problem:
    // /usr/local/include/harfbuzz/hb-common.h:68:10: fatal error: 'stddef.h' file not found
    if cfg!(not(target_os = "windows")) {
        if let Ok(output) = Command::new("gcc").arg("-print-file-name=include").output() {
            if output.status.success() {
                if let Ok(path) = String::from_utf8(output.stdout) {
                    let trimmed = path.trim();
                    clang_args.push(format!("-I{}", trimmed));
                }
            }
        }
    }

    // Add the HarfBuzz include and common system paths
    clang_args.push(format!("-I{}", hb_include));
    clang_args.push("-I/usr/include".to_string());

    let bindings = bindgen::Builder::default()
        .clang_args(clang_args)
        .header(format!("{}/hb.h", hb_include))
        .header(format!("{}/hb-subset.h", hb_include))
        .generate()
        .expect("Unable to generate bindings");    

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
