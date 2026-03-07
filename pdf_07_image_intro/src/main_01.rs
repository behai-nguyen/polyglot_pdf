/* Date Created: 28/02/2026 */

//! Render a PNG image as is onto a PDF, and write some text.

use std::fs::File;
use cairo::{Context, ImageSurface, PdfSurface};
use pango::FontDescription;
use pangocairo::functions::{create_layout, show_layout};

mod page_geometry;
use page_geometry::{
    a4_default_content_width,
    a4_default_content_height,
    A4_DEFAULT_MARGINS,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (font_description, pdf_file_name) = if cfg!(target_os = "windows") {
        ("Arial Unicode MS", "win_output_01.pdf")
    } else {
        ("NotoSansTC-Regular", "ubuntu_output_01.pdf")
    };

    let surface = PdfSurface::new(a4_default_content_width(), 
        a4_default_content_height(), pdf_file_name)?;

    let context = Context::new(&surface)?;

    // Reserve the entire context. Painting an image will alter some context information.
    context.save().expect("Failed to save cairo context");

    // Define the input PNG image file name (ensure this file exists).
    let png_file_name = "./img/139015.png"; 
    // Load the PNG image into an ImageSurface.
    // The cairo library provides a function for this, accessible via the Rust bindings.
    let mut img_file = File::open(png_file_name)?;
    let image_surface = ImageSurface::create_from_png(&mut img_file)
        .map_err(|e| format!("Failed to create image surface from PNG: {}", e))?;

    // Draw the Image onto the PDF Surface:
    // Set the image surface as the source pattern for drawing
    // Draw at position (A4_DEFAULT_MARGINS.left, A4_DEFAULT_MARGINS.top).
    context.set_source_surface(&image_surface, A4_DEFAULT_MARGINS.left, A4_DEFAULT_MARGINS.top)?;

    // Paint the source surface onto the current target surface (the PDF surface).
    context.paint()?;

    // Restore the original context.
    context.restore().expect("Failed to restore cairo context");

    // Add some text text
    let layout = create_layout(&context);
    layout.set_width((a4_default_content_width() * pango::SCALE as f64) as i32);
    let desc = FontDescription::from_string(font_description);
    layout.set_font_description(Some(&desc));

    layout.set_text("Hello, Cairo PDF with PNG!");

    context.move_to(A4_DEFAULT_MARGINS.left, A4_DEFAULT_MARGINS.top);

    // context.set_source_rgb(1.0, 0.0, 0.0); // or any color...

    show_layout(&context, &layout);

    // show_page() finishes the current page and commits pending drawing operations.
    context.show_page()?;

    // Finish the surface to ensure all data is written to the file stream.
    surface.finish();

    println!("Successfully generated PDF: {}", pdf_file_name);

    Ok(())
}