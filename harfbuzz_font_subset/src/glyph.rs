// 03/11/2025

use std::{ptr, slice};
use std::collections::HashSet;

#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod hb {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use hb::*;

pub fn get_glyph_ids(font_data: &[u8], face_index: u32, text: &str) -> Result<Vec<u32>, String> {
    // Create blob and face
    let blob = unsafe {
        hb_blob_create(
            font_data.as_ptr() as *const i8,
            font_data.len() as u32,
            hb_memory_mode_t_HB_MEMORY_MODE_READONLY,
            ptr::null_mut(),
            None,
        )
    };
    let face = unsafe { hb_face_create(blob, face_index) };
    let font = unsafe { hb_font_create(face) };

    // Create buffer and add text
    let buffer = unsafe { hb_buffer_create() };
    let c_str = std::ffi::CString::new(text).unwrap();
    unsafe {
        hb_buffer_add_utf8(buffer, c_str.as_ptr(), -1, 0, -1);
        hb_buffer_guess_segment_properties(buffer);
        hb_shape(font, buffer, ptr::null(), 0);
    }

    // Extract unique glyph infos
    let mut length: u32 = 0;
    let infos = unsafe { hb_buffer_get_glyph_infos(buffer, &mut length) };
    let glyphs = unsafe { slice::from_raw_parts(infos, length as usize) }
        .iter()
        .map(|info| info.codepoint)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<u32>>();

    // Cleanup
    unsafe {
        hb_buffer_destroy(buffer);
        hb_font_destroy(font);
        hb_face_destroy(face);
        hb_blob_destroy(blob);
    }

    Ok(glyphs)
}