// 22/11/2025

use std::process;

#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod hb {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use crate::hb::{
    hb_direction_t_HB_DIRECTION_LTR,
    hb_script_t_HB_SCRIPT_LATIN,
};

mod pdf_font_info;
mod subset_builder;
mod page_geometry;
mod pdf_text;
mod text_layout;
mod pdf_gen;

use pdf_font_info::{FontInfo, get_font_info};
use subset_builder::get_font_subset;
use page_geometry::{
    A4, 
    a4_default_content_width, 
    a4_default_content_height, 
    A4_DEFAULT_MARGINS, 
    LINE_SPACING_FACTOR
};
use pdf_text::PdfTextContent;
use text_layout::text_to_lines;
use pdf_gen::generate_pdf;

fn main() {
    let input_text_file = "./text/essay.txt";

    let (input_font_name, pdf_file_name) = if cfg!(target_os = "windows") {
        ("Arial Unicode MS", "win_lopdf.pdf")
    } else {
        ("NotoSansTC-Regular", "ubuntu_lopdf.pdf")
    };

    let mut pdf_text = PdfTextContent::new(hb_direction_t_HB_DIRECTION_LTR, 
        hb_script_t_HB_SCRIPT_LATIN, "vi", 12.0);

    match pdf_text.prepare(input_text_file, input_font_name) {
        Ok(_) => {},
        Err(err) => {
            println!("{err}");
            process::exit(1);
        }
    }

    match generate_pdf(&pdf_text, pdf_file_name) {
            Ok(_) => {},
            Err(err) => println!("Error: {err}"),
    }
}