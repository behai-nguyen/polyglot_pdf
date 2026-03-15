/* 09/03/2026. */

use cairo::{Context, PdfSurface};

mod text_layout;

mod page_geometry;
use page_geometry::{A4, A4_DEFAULT_MARGINS};

mod config;
use config::load_config;

mod document;

mod font_utils;

mod image_layout;
use image_layout::layout_image_block;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (config_file_name, pdf_file_name) = if cfg!(target_os = "windows") {
        ("./config/config.toml", "win_image_block.pdf")
    } else {
        ("./config/config.toml", "ubuntu_image_block.pdf")
    };

	let config = load_config(config_file_name)?;	

    let surface = PdfSurface::new(A4.width, A4.height, pdf_file_name)?;

    let context = Context::new(&surface)?;

    let caption: &str = "Cassowary, an Australia native, and “the world's most dangerous bird”.\n\
        Cassowary, chim bản địa Úc, và là “loài chim nguy hiểm nhất thế giới”.";
    // Define the input PNG image file name (ensure this file exists).
    let png_file_name = "./img/139015.png";
    let image_bottom = layout_image_block(png_file_name, caption, 
        // A4_DEFAULT_MARGINS.top, 
        200.00,
        &context, &config)?;

    let caption: &str = "Can Roadrunners outrun Cassowaries?";
    // Define the input PNG image file name (ensure this file exists).
    let png_file_name = "./img/KTmCgCBjQXKLsO2JeBMVrA.png";
    let _image_bottom = layout_image_block(png_file_name, caption,
        image_bottom, 
        &context, &config)?;

    context.show_page()?;

    // Finish the surface to ensure all data is written to the file stream.
    surface.finish();

    println!("Successfully generated PDF: {}", pdf_file_name);

	Ok(())
}