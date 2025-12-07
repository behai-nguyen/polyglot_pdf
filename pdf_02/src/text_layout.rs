// 23/11/2025

/// Breaking a paragraph into lines which fit into a predefined page width.

use std::ptr;
use std::ffi::CString;

use crate::hb::{
    hb_font_t,
    hb_buffer_t,
    hb_glyph_position_t,
    hb_memory_mode_t_HB_MEMORY_MODE_READONLY,
    hb_direction_t,
    hb_script_t,
    hb_blob_create,
    hb_face_create,
    hb_font_create,
    hb_buffer_create,
    hb_buffer_clear_contents,
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

/// Calculate a token width in PostScript point.
fn width_in_point(token: &str,
    hb_direction: hb_direction_t,
    hb_script: hb_script_t,
    language: &str,
    font: *mut hb_font_t,
    buffer: *mut hb_buffer_t,
    scale: f32
) -> f32 {
    let mut width_in_pt: f32 = 0.0;

    unsafe {
        // Add text to buffer
        hb_buffer_add_utf8(buffer, CString::new(token).unwrap().as_ptr(), 
            -1, 0, -1);
        hb_buffer_set_direction(buffer, hb_direction);
        hb_buffer_set_script(buffer, hb_script);
        hb_buffer_set_language(buffer, hb_language_from_string(
            CString::new(language).unwrap().as_ptr(), -1));

        // Shape the text
        hb_shape(font, buffer, std::ptr::null(), 0);

        // Get glyph information (positions and glyph IDs)        
        let mut glyph_count: u32 = 0;
        let glyph_pos = hb_buffer_get_glyph_positions(buffer, &mut glyph_count);
        for i in 0..glyph_count as usize {
            let pos = get_glyph_pos(glyph_pos, i);
            width_in_pt += pos.x_advance as f32 * scale;
        }
    }

    width_in_pt
}

/// Breaking a paragraph into lines which fit into a predefined page width.
/// This is a simple greedy line breaking algorithm.
pub fn text_to_lines(text: &str, 
    page_width: f32,
    hb_direction: hb_direction_t,
    hb_script: hb_script_t,
    language: &str,
    font_data: &[u8], 
    scale: f32
) -> Vec<String> {
    // Load a font blob
    let blob = unsafe {
        hb_blob_create(
            font_data.as_ptr() as *const i8,
            font_data.len() as u32,
            hb_memory_mode_t_HB_MEMORY_MODE_READONLY,
            ptr::null_mut(),
            None,
        )
    };

    // Create a face from the blob
    let face = unsafe { hb_face_create(blob, 0) };

    // Create a font from the face
    let font = unsafe { hb_font_create(face) };

    // Create a buffer for text shaping
    let buffer = unsafe { hb_buffer_create() };

    // The width of the space for the selected font
    let space_width_in_pt: f32 = width_in_point(" ", hb_direction, hb_script, 
        language, font, buffer, scale);
    unsafe { hb_buffer_clear_contents(buffer); }

    let mut result: Vec<String> = Vec::new();

    // Split by any common newline sequence and keep the delimiters
    let lines_vec: Vec<String> = text
        .split_inclusive('\n') // Splits and includes the '\n'
        .map(|s| s.to_string())
        .collect();

    for line in lines_vec {        
        if line == "\n" || line == "\r\n" {
            result.push(" ".to_string());
            continue;
        }

        let mut shaped_words: Vec<(String, f32)> = Vec::new();

        let words: Vec<&str> = line.split_whitespace().collect();

        for word in words {
            let width_in_pt: f32 = width_in_point(word, hb_direction, hb_script, 
                language, font, buffer, scale);
            shaped_words.push((word.to_string(), width_in_pt));
            unsafe { hb_buffer_clear_contents(buffer); }
        }

        let mut current_line = String::new();
        let mut current_width = 0.0;

        for (word, width) in shaped_words {
            if current_width + width + space_width_in_pt > page_width {
                result.push(current_line.trim().to_string());
                current_line.clear();
                current_width = 0.0;
            }
            current_line.push_str(&word);
            current_line.push(' ');
            current_width += width + space_width_in_pt;
        }
        if !current_line.is_empty() {
            result.push(current_line.trim().to_string());
        }
    }

    // Clean up resources
    unsafe {
        hb_buffer_destroy(buffer);
        hb_font_destroy(font);
        hb_face_destroy(face);
        hb_blob_destroy(blob);
    };

    result
}