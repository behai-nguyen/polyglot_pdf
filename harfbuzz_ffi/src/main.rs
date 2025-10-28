// 26/10/2025

use std::ffi::CStr;

#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod hb {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[link(name = "harfbuzz")]
unsafe extern "C" {
    pub fn hb_version_string() -> *const std::os::raw::c_char;
}

fn main() {
    unsafe {
        let version_ptr = hb::hb_version_string();
        let version = CStr::from_ptr(version_ptr);
        println!("HarfBuzz version: {}", version.to_str().unwrap());
    }
}
