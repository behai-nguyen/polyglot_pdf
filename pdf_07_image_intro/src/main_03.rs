/* Date Created: 02/03/2026 */

//! Scale PNG image by the width to fit across the page.
//! Render the scaled image onto a PDF:
//!     1. A4.width, A4.height
//!     2. No context.translate(): uses original_factor.
//!     3. context.save() and context.restore().
//! 
//! Not recommended, because:
//! 
//!     1. **Translate first, then scale**, not “scale and compensate.”
//!     2. “scale then compensate” approach is mathematically fine, but it 
//!        goes *against* the grain of how Cairo expects developers to think.

use std::{fs::File, ops::Mul};
use cairo::{Context, ImageSurface, PdfSurface};

mod page_geometry;
use page_geometry::{
    A4,
    a4_default_content_width,
    A4_DEFAULT_MARGINS,
};

fn get_scaling_factor(img_surface: &ImageSurface) -> f64 {
    a4_default_content_width() / img_surface.width() as f64
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_font_description, pdf_file_name) = if cfg!(target_os = "windows") {
        ("Arial Unicode MS", "win_output_03.pdf")
    } else {
        ("NotoSansTC-Regular", "ubuntu_output_03.pdf")
    };

    let surface = PdfSurface::new(A4.width, A4.height, pdf_file_name)?;

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

    let scale_factor = get_scaling_factor(&image_surface);

    // Apply scale transformation
    context.scale(scale_factor, scale_factor); 

    let original_factor: f64 = 1.0 / scale_factor;

    // Draw the Image onto the PDF Surface:
    // Set the image surface as the source pattern for drawing
    // Draw the image at the original scale (A4_DEFAULT_MARGINS.left, A4_DEFAULT_MARGINS.top).
    context.set_source_surface(&image_surface, A4_DEFAULT_MARGINS.left.mul(original_factor), 
        A4_DEFAULT_MARGINS.top.mul(original_factor))?;

    // Paint the source surface onto the current target surface (the PDF surface).
    context.paint()?;

    // Restore the original context.
    context.restore().expect("Failed to restore cairo context");     

    // show_page() finishes the current page and commits pending drawing operations.
    context.show_page()?;

    // Finish the surface to ensure all data is written to the file stream.
    surface.finish();

    println!("Successfully generated PDF: {}", pdf_file_name);

    Ok(())
}