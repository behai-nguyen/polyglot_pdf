// 29/10/2025

use std::process;

#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod hb {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

mod pdf_font_info;
mod subset_builder;
mod pdf_page;
mod pdf_gen;

use pdf_font_info::get_font_info;
use subset_builder::get_font_subset;
use pdf_page::{PdfPage, PdfPages};
use pdf_gen::generate_pdf;

fn main() {
    let (input_font_name, pdf_file_name) = if cfg!(target_os = "windows") {
        ("Arial Unicode MS", "win_lopdf.pdf")
    } else {
        ("NotoSansTC-Regular", "ubuntu_lopdf.pdf")
    };

    // 1) Font program location, face index, embeded name
    let font_info = match get_font_info(input_font_name) {
        Some(fi) => fi,
        None => {
            println!("\nFont: {} not found.", input_font_name);
            process::exit(1);
        },        
    };

    println!("path: {}, face index: {}, embedded name: {}", font_info.path, font_info.face_index, font_info.embedded_name);

    let mut pdf_pages = PdfPages::new()
        .add_page("Kỷ độ Long Tuyền đới nguyệt ma.")
        .add_page("幾度龍泉戴月磨。");

    /* 
    let mut pdf_pages = PdfPages::new()
        .add_page("Kỷ độ Long Tuyền đới nguyệt ma.")
        .add_page("幾度龍泉戴月磨。")
        .add_page("— General Đặng Dung, Later Trần Dynasty.");
    */

    // 2) Font subset for all text in PDF document
    let font_subset = match get_font_subset(
        &font_info.path, 
        font_info.face_index, 
        &pdf_pages.all_text()
    ) {
        Ok(result) => result,
        Err(err) => {
            println!("{err}");
            process::exit(2);
        }
    };

    // 3) Generat PDF
    match generate_pdf(&font_subset, 
        &font_info.embedded_name,
        &mut pdf_pages,
        pdf_file_name) {
            Ok(_) => {},
            Err(err) => println!("Error: {err}"),
    }
}