// 22/11/2025
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
    hb_font_t,
    hb_buffer_t,
    hb_glyph_position_t,
    hb_memory_mode_t_HB_MEMORY_MODE_READONLY,
    hb_direction_t_HB_DIRECTION_LTR,
    hb_script_t_HB_SCRIPT_LATIN,
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

fn width_in_point(token: &str,
    lang: &str,
    font: *mut hb_font_t,
    buffer: *mut hb_buffer_t,
    scale: f32
) -> f32 {
    let mut width_in_pt: f32 = 0.0;

    unsafe {
        // 4. Add text to it
        hb_buffer_add_utf8(buffer, CString::new(token).unwrap().as_ptr(), -1, 0, -1);
        hb_buffer_set_direction(buffer, hb_direction_t_HB_DIRECTION_LTR);
        hb_buffer_set_script(buffer, hb_script_t_HB_SCRIPT_LATIN);
        hb_buffer_set_language(buffer, hb_language_from_string(CString::new(lang).unwrap().as_ptr(), -1));

        // 5. Shape the text
        hb_shape(font, buffer, std::ptr::null(), 0);

        // 6. Get glyph information (positions and glyph IDs)        
        let mut glyph_count: u32 = 0;
        let glyph_pos = hb_buffer_get_glyph_positions(buffer, &mut glyph_count);
        for i in 0..glyph_count as usize {
            let pos = get_glyph_pos(glyph_pos, i);
            width_in_pt += pos.x_advance as f32 * scale;
        }
    }

    width_in_pt
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

    let text = "Lịch sử Việt Nam từ năm 1945 đến nay, còn nhiều bí ẩn chưa được giải tỏa. Người bàng quan, các thế hệ sau, sẽ không thấy được những âm mưu thầm kín của ông Hồ đã tiêu diệt người quốc gia, nếu như chúng ta không phát hiện được những bí mật lịch sử đó. Chúng tôi may mắn được nhà sử học Chính Ðạo, tức tiến sĩ Vũ Ngự Chiêu, cho phép sử dụng nhiều tài liệu quý giá mà ông sao lục từ các văn khố, thư viện của bộ Thuộc Ðịa, bộ Ngoại Giao Pháp… để làm sáng tỏ nhiều uẩn khúc lịch sử, vốn bị cộng sản che giấu, nhiễu loạn từ hơn nửa thế kỷ qua. Chúng tôi chân thành cảm tạ tiến sĩ Chiêu. Trong loạt bài nầy, chúng tôi sẽ trưng bằng chứng về những hành vi phản bội quyền lợi dân tộc của ông Hồ. Nổi thao thức của ông Hồ lúc nầy là Việt Minh phải mắm chính quyền, không chia xẻ, nhượng bộ cho bất cứ đảng phái nào. Ðó là đường lối nhất quán, trước sau như một của đảng cộng sản. Ðây cũng là dự mưu, từ khi ngoài rừng núi Tân Trào kéo về Hà Nội. “Căn cứ vào kết quả của cuộc thảo luận của ông Hồ cùng các cán bộ, thấy rằng công cuộc phát triển cách mạng của họ sẽ dẫn đến 2 trường hợp:";
    let lang = "vi";

    // 4. Create a buffer and add text to it
    let buffer = unsafe { hb_buffer_create() };

    let space_width_in_pt: f32 = width_in_point(" ", lang, font, buffer, scale);
    unsafe { hb_buffer_clear_contents(buffer); }

    let words: Vec<&str> = text.split_whitespace().collect();

    let mut shaped_words: Vec<(&str, f32)> = Vec::new();
    for word in words {
        let width_in_pt: f32 = width_in_point(word, lang, font, buffer, scale);
        shaped_words.push((word, width_in_pt));
        unsafe { hb_buffer_clear_contents(buffer); }
    }

    // 7. Clean up resources
    unsafe {
        hb_buffer_destroy(buffer);
        hb_font_destroy(font);
        hb_face_destroy(face);
        hb_blob_destroy(blob);
    };

    // Margin: 20mm. 20 x (72 / 25.4) = 57 Postscript point.
    let margin: f32 = 57.0;
    let a4_width: f32 = 595.22 - margin - margin;

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0.0;

    for (word, width) in shaped_words {
        if current_width + width + space_width_in_pt > a4_width {
            lines.push(current_line.trim().to_string());
            current_line.clear();
            current_width = 0.0;
        }
        current_line.push_str(word);
        current_line.push(' ');
        current_width += width + space_width_in_pt;
    }
    if !current_line.is_empty() {
        lines.push(current_line.trim().to_string());
    }

    for line in lines {
        println!("{}", line);
    }
}
