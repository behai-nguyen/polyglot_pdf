// 09/12/2025

fn main() {
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-search=native=C:/PF/pango/dist/lib");
        println!("cargo:rustc-link-search=native=C:/PF/cairo-1.18.4/dist/lib");
    };

    println!("cargo:rustc-link-lib=pango-1.0");
    println!("cargo:rustc-link-lib=pangocairo-1.0");
    println!("cargo:rustc-link-lib=cairo");
}