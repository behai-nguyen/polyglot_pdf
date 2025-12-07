// 12/11/2025
// Based on Google AI Overview example in C when search with: "HarfBuzz text shaping example"
//    https://www.google.com/search?q=HarfBuzz+text+shaping+example&sca_esv=2286498253ea9ec4&sxsrf=AE3TifOaqxgXsmoib3nw-seZgGY94QlA4Q%3A1762907608173&ei=2NUTae6rCr-dseMPoor7kAs&ved=0ahUKEwju94bOruuQAxW_TmwGHSLFHrIQ4dUDCBE&uact=5&oq=HarfBuzz+text+shaping+example&gs_lp=Egxnd3Mtd2l6LXNlcnAiHUhhcmZCdXp6IHRleHQgc2hhcGluZyBleGFtcGxlMgUQIRigATIFECEYoAFI2xtQgAVYuhpwAXgBkAEAmAH-AaABvAqqAQUwLjcuMbgBA8gBAPgBAZgCCaACgAvCAgoQABiwAxjWBBhHwgIGEAAYFhgewgIFEAAY7wXCAggQABiABBiiBMICBBAhGBXCAgcQIRigARgKmAMAiAYBkAYIkgcFMS43LjGgB9AUsgcFMC43LjG4B_gKwgcHMC40LjQuMcgHIw&sclient=gws-wiz-serp

use std::{fs, ptr};
use ttf_parser::Face;
use std::ffi::CString;

#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod hb {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use crate::hb::{
    hb_glyph_position_t,
    hb_memory_mode_t_HB_MEMORY_MODE_READONLY,
    hb_direction_t_HB_DIRECTION_LTR,
    hb_script_t_HB_SCRIPT_LATIN,
    hb_blob_create,
    hb_face_create,
    hb_font_create,
    hb_buffer_create,
    hb_buffer_add_utf8,
    hb_buffer_set_direction,
    hb_buffer_set_script,
    hb_language_from_string,
    hb_buffer_set_language,
    hb_shape,
    hb_buffer_get_glyph_positions,
    hb_buffer_destroy,
    hb_font_destroy,
    hb_face_destroy,
    hb_blob_destroy,
};

unsafe fn get_glyph_pos(pos: *const hb_glyph_position_t, i: usize) -> hb_glyph_position_t {
    unsafe { *pos.add(i) }
}

fn main() {
    let input_font_file = if cfg!(target_os = "windows") {
        "C:/Windows/Fonts/arialuni.ttf"
    } else {
        "/home/behai/Noto_Sans_TC/NotoSansTC-Regular.ttf"
    };

    // Load font file into memory
    let font_data = fs::read(input_font_file).expect("cannot read font");

    // Parse with ttf-parser
    let face = Face::parse(&font_data, 0).expect("bad font");
    // Collect advances (in font units)
    let units_per_em = face.units_per_em() as f32;

    let font_size_pt = 12.0; // desired size in PostScript points

    let scale = font_size_pt / units_per_em;

    let mut total_advance = 0.0;

    // 1. Load a font blob
    let blob = unsafe {
        hb_blob_create(
            font_data.as_ptr() as *const i8,
            font_data.len() as u32,
            hb_memory_mode_t_HB_MEMORY_MODE_READONLY,
            ptr::null_mut(),
            None,
        )
    };

    // 2. Create a face from the blob
    let face = unsafe { hb_face_create(blob, 0) };

    // 3. Create a font from the face
    let font = unsafe { hb_font_create(face) };

    let text = "Kỷ độ Long Tuyền đới nguyệt ma.";
    let lang = "vi";

    // 4. Create a buffer and add text to it
    let buffer = unsafe { hb_buffer_create() };
    unsafe { hb_buffer_add_utf8(buffer, 
        CString::new(text).unwrap().as_ptr(), -1, 0, -1) 
    };

    unsafe { 
        // Set buffer properties (optional, but important for correct shaping)
        hb_buffer_set_direction(buffer, hb_direction_t_HB_DIRECTION_LTR); // Left-to-right
        hb_buffer_set_script(buffer, hb_script_t_HB_SCRIPT_LATIN);     // Latin script
        hb_buffer_set_language(buffer, 
            hb_language_from_string(CString::new(lang).unwrap().as_ptr(), -1)); // Vietnamese
    };

    // 5. Shape the text
    unsafe { hb_shape(font, buffer, std::ptr::null(), 0) };

    // 6. Get glyph information (positions and glyph IDs)
    let mut glyph_count: u32 = 0;
    let glyph_pos = unsafe { hb_buffer_get_glyph_positions(buffer, &mut glyph_count) };

    for i in 0..glyph_count as usize {
        let pos = unsafe { get_glyph_pos(glyph_pos, i) };

        total_advance += pos.x_advance as f32 * scale;
    }

    println!(
        "\"{}\" in {} pt {} is {:.2} pt wide",
        text, font_size_pt, input_font_file, total_advance
    );

    // 7. Clean up resources
    unsafe {
        hb_buffer_destroy(buffer);
        hb_font_destroy(font);
        hb_face_destroy(face);
        hb_blob_destroy(blob);
    };
}