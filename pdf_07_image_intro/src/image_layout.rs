/* Date Created: 03/03/2026 */

//! 
//! 1. Render the scaled image onto a PDF:
//!     a. A4.width, A4.height
//!     b. context.translate() before context.scale().
//!     c. context.save() and context.restore().
//! 

use std::{fs::File};
use cairo::{Context, ImageSurface};

use crate::page_geometry::{
    a4_default_content_width,
    A4_DEFAULT_MARGINS,
};

/// Scale an already loaded image to fit [`a4_default_content_width()`].
/// 
/// If original image width is less than [`a4_default_content_width()`], the returned 
/// factor is greater than 1.0, representing a scale up, the quality of the rendered 
/// image might not be as good as the original image.
/// 
/// If the original image width is greater than [`a4_default_content_width()`], the 
/// returned factor is less than 1.0, representing a scale down.
/// 
/// # Arguments
/// 
/// * `img_surface` - [`ImageSurface`], an already loaded image.
/// 
/// # Return
/// 
/// [`f64`] - the image scaling factor.
/// 
fn get_scaling_factor(img_surface: &ImageSurface) -> f64 {
    a4_default_content_width() / img_surface.width() as f64
}

/// Determine the scale factor to fit the image from `image_file_name` to 
/// [`a4_default_content_width()`].
/// 
/// Then apply `reduction_factor` to scale factor to calculate the final scale factor.
/// 
/// Use the final scale factor to scale the image. I.e. the image is scaled only once.
/// 
/// # Arguments
/// 
/// * `image_file_name` — path to the image file.
/// 
/// * `reduction_factor` — additional scaling applied after fitting the image to
///   the page width (e.g. `0.1` for a further 10% reduction).
/// 
/// * `centre_aligned` — whether to horizontally center the image and caption.
///   Only meaningful when the scaled image is narrower than the page width.
/// 
/// * `top_y` — the y‑coordinate at which to place the top of the image.
/// 
/// * `context` — the Cairo PDF [`Context`].
/// 
/// # Return
/// 
/// * `f64` — on success, the y‑coordinate of the bottom of the rendered image.
///   This can be used to position subsequent content.
/// 
/// * `std::error::Error` — if the image cannot be fitted even after progressive
///   reduction and a page break.
/// 
pub fn render_image_block(image_file_name: &str, 
    reduction_factor: f64, 
    centre_aligned: bool,
    top_y: f64, 
    context: &Context
) -> Result<f64, Box<dyn std::error::Error>> {
    // Reserve the entire context. Painting an image will alter some context information.
    context.save().expect("Failed to save cairo context");

    // Load the PNG image into an ImageSurface.
    // The cairo library provides a function for this, accessible via the Rust bindings.
    let mut img_file = File::open(image_file_name)?;
    let image_surface = ImageSurface::create_from_png(&mut img_file)
        .map_err(|e| format!("Failed to create image surface from PNG: {}", e))?;

    let scale_factor = get_scaling_factor(&image_surface) * reduction_factor;

    let x: f64 = if centre_aligned {
        let width: f64 = image_surface.width() as f64 * scale_factor;
        ( (a4_default_content_width() - width) / 2.0 ) + A4_DEFAULT_MARGINS.left
    } else { A4_DEFAULT_MARGINS.left };

    // Move to the top-left content area (unscaled)
    context.translate(x, top_y);
    
    // Apply scale transformation
    context.scale(scale_factor, scale_factor); 

    // Draw the Image onto the PDF Surface:
    // Set the image surface as the source pattern for drawing
    // Draw the image at (0, 0) in scaled coordinates.
    context.set_source_surface(&image_surface, 0.0, 0.0)?;

    // Paint the source surface onto the current target surface (the PDF surface).
    context.paint()?;

    // Restore the original context.
    context.restore().expect("Failed to restore cairo context");

    // Effectively the height of the scaled image.
    let image_bottom: f64 = top_y + (image_surface.height() as f64 * scale_factor);

    Ok(image_bottom)
}
