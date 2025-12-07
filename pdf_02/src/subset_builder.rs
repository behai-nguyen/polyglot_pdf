// 30/10/2025

use std::{fs, ptr, slice};

use crate::hb::{
    hb_memory_mode_t_HB_MEMORY_MODE_READONLY,
    hb_blob_create,
    hb_face_create,
    hb_subset_input_create_or_fail,
    hb_subset_input_unicode_set,
    hb_set_add,
    hb_subset_or_fail,
    hb_face_reference_blob,
    hb_blob_get_data,
    hb_blob_destroy,
    hb_face_destroy,
    hb_subset_input_destroy,
};

pub fn get_font_subset(input_font_file: &str, 
    face_index: u32, 
    text: &str
) -> Result<Vec<u8>, String> {
    // Load font file
    let font_data = fs::read(input_font_file).map_err(|e| e.to_string())?;

    // Create blob
    let blob = unsafe {
        hb_blob_create(
            font_data.as_ptr() as *const i8,
            font_data.len() as u32,
            hb_memory_mode_t_HB_MEMORY_MODE_READONLY,
            ptr::null_mut(),
            None,
        )
    };

    // Create face
    let face = unsafe { hb_face_create(blob, face_index) };

    // Create subset input
    let input = unsafe { hb_subset_input_create_or_fail() };
    let unicode_set = unsafe { hb_subset_input_unicode_set(input) };

    for cp in text.chars() {
        unsafe { hb_set_add(unicode_set, cp as u32) };
    }

    // Subset
    let subset_face = unsafe { hb_subset_or_fail(face, input) };
    if subset_face.is_null() {
        return Err("hb_subset_or_fail returned null".into());
    }

    // Get blob
    let subset_blob = unsafe { hb_face_reference_blob(subset_face) };
    if subset_blob.is_null() {
        return Err("hb_face_reference_blob returned null".into());
    }

    // Copy data
    let mut length: u32 = 0;
    let data_ptr = unsafe { hb_blob_get_data(subset_blob, &mut length) };
    if data_ptr.is_null() || length == 0 {
        return Err("hb_blob_get_data returned null or empty".into());
    }

    let slice = unsafe { slice::from_raw_parts(data_ptr as *const u8, length as usize) };
    let result = slice.to_vec(); // ✅ Rust-owned copy

    // Cleanup
    unsafe {
        hb_blob_destroy(blob);
        hb_face_destroy(face);
        hb_subset_input_destroy(input);
        hb_face_destroy(subset_face);
        hb_blob_destroy(subset_blob);
    }

    Ok(result)
}