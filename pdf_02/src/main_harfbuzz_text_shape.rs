// 12/11/2025
// Based on Google AI Overview example in C when search with: "HarfBuzz text shaping example"
//    https://www.google.com/search?q=HarfBuzz+text+shaping+example&sca_esv=2286498253ea9ec4&sxsrf=AE3TifOaqxgXsmoib3nw-seZgGY94QlA4Q%3A1762907608173&ei=2NUTae6rCr-dseMPoor7kAs&ved=0ahUKEwju94bOruuQAxW_TmwGHSLFHrIQ4dUDCBE&uact=5&oq=HarfBuzz+text+shaping+example&gs_lp=Egxnd3Mtd2l6LXNlcnAiHUhhcmZCdXp6IHRleHQgc2hhcGluZyBleGFtcGxlMgUQIRigATIFECEYoAFI2xtQgAVYuhpwAXgBkAEAmAH-AaABvAqqAQUwLjcuMbgBA8gBAPgBAZgCCaACgAvCAgoQABiwAxjWBBhHwgIGEAAYFhgewgIFEAAY7wXCAggQABiABBiiBMICBBAhGBXCAgcQIRigARgKmAMAiAYBkAYIkgcFMS43LjGgB9AUsgcFMC43LjG4B_gKwgcHMC40LjQuMcgHIw&sclient=gws-wiz-serp

use std::ffi::CString;

#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod hb {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use crate::hb::{
    hb_glyph_info_t,
    hb_glyph_position_t,
    hb_direction_t_HB_DIRECTION_LTR,
    hb_script_t_HB_SCRIPT_LATIN,
    hb_blob_create_from_file,
    hb_face_create,
    hb_font_create,
    hb_buffer_create,
    hb_buffer_add_utf8,
    hb_buffer_set_direction,
    hb_buffer_set_script,
    hb_language_from_string,
    hb_buffer_set_language,
    hb_shape,
    hb_buffer_get_glyph_infos,
    hb_buffer_get_glyph_positions,
    hb_buffer_destroy,
    hb_font_destroy,
    hb_face_destroy,
    hb_blob_destroy,
};

unsafe fn get_glyph_info(info: *const hb_glyph_info_t, i: usize) -> hb_glyph_info_t {
    unsafe { *info.add(i) }
}

unsafe fn get_glyph_pos(pos: *const hb_glyph_position_t, i: usize) -> hb_glyph_position_t {
    unsafe { *pos.add(i) }
}

fn main() {
    let input_font_file = if cfg!(target_os = "windows") {
        "C:/Windows/Fonts/arialuni.ttf"
    } else {
        "/home/behai/Noto_Sans_TC/NotoSansTC-Regular.ttf"
    };    

    // 1. Load a font blob
    let blob = unsafe { 
        hb_blob_create_from_file(CString::new(input_font_file)
            .expect("Failed font file name").as_ptr()) 
    };

    // 2. Create a face from the blob
    let face = unsafe { hb_face_create(blob, 0) };

    // 3. Create a font from the face
    let font = unsafe { hb_font_create(face) };

    // let text = "Hello World!";
    // let lang = "en";

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
    let glyph_info = unsafe { hb_buffer_get_glyph_infos(buffer, &mut glyph_count) };
    let glyph_pos = unsafe { hb_buffer_get_glyph_positions(buffer, &mut glyph_count) };

    println!("Shaped text glyph information:");
    /*
    for i in 0..glyph_count {
        println!("Glyph ID: {}, Cluster: {}, X Advance: {}, Y Advance: {}, X Offset: {}, Y Offset: {}",
            unsafe { (*glyph_info.wrapping_add(i as usize)).codepoint },
            unsafe { (*glyph_info.wrapping_add(i as usize)).cluster },
            unsafe { (*glyph_pos.wrapping_add(i as usize)).x_advance },
            unsafe { (*glyph_pos.wrapping_add(i as usize)).y_advance },
            unsafe { (*glyph_pos.wrapping_add(i as usize)).x_offset },
            unsafe { (*glyph_pos.wrapping_add(i as usize)).y_offset });
    }
    */
    for i in 0..glyph_count as usize {
        let info = unsafe { get_glyph_info(glyph_info, i) };
        let pos = unsafe { get_glyph_pos(glyph_pos, i) };

        println!(
            "Glyph ID: {}, Cluster: {}, X Advance: {}, Y Advance: {}, X Offset: {}, Y Offset: {}",
            info.codepoint, info.cluster, pos.x_advance, pos.y_advance, pos.x_offset, pos.y_offset
        );
    }

    // 7. Clean up resources
    unsafe {
        hb_buffer_destroy(buffer);
        hb_font_destroy(font);
        hb_face_destroy(face);
        hb_blob_destroy(blob);
    };
}