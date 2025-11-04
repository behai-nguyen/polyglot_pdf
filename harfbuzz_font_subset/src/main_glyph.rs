// 29/10/2025

use std::{fs, process, ptr};
use std::slice;

mod glyph;
use glyph::get_glyph_ids;

#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod hb {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use crate::hb::{
    hb_memory_mode_t_HB_MEMORY_MODE_READONLY,
    hb_blob_create,
    hb_face_create,
    hb_subset_input_create_or_fail,
    hb_subset_input_set_flags,
    hb_subset_input_glyph_set,
    hb_set_add,    
    hb_subset_or_fail,
    hb_face_reference_blob,
    hb_blob_get_data,
    hb_blob_destroy,
    hb_face_destroy,
    hb_subset_input_destroy,
};

fn main() {
    let (input_font_file, output_font_file) = if cfg!(target_os = "windows") {
        (
            "C:/Windows/Fonts/arialuni.ttf",
            "win_subset.ttf",
        )        
    } else {
        (
            "/home/behai/Noto_Sans_TC/NotoSansTC-Regular.ttf",
            "linux_subset.ttf",
        )
    };

    let font_data = fs::read(input_font_file).unwrap();

    let text = "Kỷ độ Long Tuyền đới nguyệt ma. 幾度龍泉戴月磨。";

    let glyph_ids = match get_glyph_ids(&font_data, 0, text) {
        Ok(result) => result,
        Err(err) => {
            println!("Error: {}", err);
            process::exit(1);
        }
    };

    // println!("glyph_ids: {:?}", glyph_ids);

    let blob = unsafe {
        hb_blob_create(font_data.as_ptr() as *const i8, 
            font_data.len() as u32, 
            hb_memory_mode_t_HB_MEMORY_MODE_READONLY, 
            ptr::null_mut(), 
            None)
    };

    let face = unsafe { hb_face_create(blob, 0) };

    let input = unsafe { hb_subset_input_create_or_fail() };

    unsafe { hb_subset_input_set_flags(input, 0) };

    // Get the glyph set owned by input
    let glyphs = unsafe { hb_subset_input_glyph_set(input) };
    for gid in glyph_ids {
        unsafe { hb_set_add(glyphs, gid) };
    };

    let subset_face = unsafe { hb_subset_or_fail(face, input) };
    let subset_blob = unsafe { hb_face_reference_blob(subset_face) };

    let mut length: u32 = 0;
    let data_ptr = unsafe { hb_blob_get_data(subset_blob, &mut length) };

    // Copy bytes
    let slice = unsafe { slice::from_raw_parts(data_ptr as *const u8, length as usize) };
    let result = slice.to_vec();

    unsafe { hb_blob_destroy(blob) };
    unsafe { hb_face_destroy(subset_face) };
    unsafe { hb_subset_input_destroy(input) };
    unsafe { hb_face_destroy(face) };

    fs::write(output_font_file, result).unwrap();

    println!("Font subset writtent to: {}", output_font_file);
}
