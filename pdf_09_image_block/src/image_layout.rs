/* Date Created: 03/03/2026 */

//! 
//! Layout an image and its associated caption onto a PDF. The layout algorithm is 
//! represented in detail in the function [`measure_image_block()`] documentation.
//! 

use cairo::{Context, ImageSurface};
use pango::Layout;
use pangocairo::functions::show_layout;

use crate::page_geometry::{
    a4_default_content_width,
    a4_default_content_height,
    A4_DEFAULT_MARGINS,
};

use crate::config::Config;

use crate::document;
use document::{ImageBlockLayoutInfo, ImageBlockMeasuredInfo};

use crate::text_layout::{
    layout_block_height,
    center_layout_block,
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
/// * `original_image_width` — physical width of the image.
/// 
/// # Return
/// 
/// [`f64`] — the image scaling factor.
/// 
fn get_scaling_factor(original_image_width: f64) -> f64 {
    a4_default_content_width() / original_image_width
}

/// This function is used by [`layout_image_block()`] to determine an appropriate
/// scale factor and vertical position for an image block (image + caption).
///
/// The caller first computes `original_scale_factor`, which scales the image to
/// fit within [`a4_default_content_width()`]. This guarantees that the image
/// fits horizontally on the page, but does *not* guarantee that the image block
/// (image + caption) fits vertically in the remaining space.
///
/// This helper attempts to find a scale factor and a vertical position (`top_y`)
/// where the entire image block fits. The algorithm is:
///
/// 1. Try placing the block at `current_top_y` on the current page.
///    If the image scaled by `original_scale_factor`, together with the caption
///    height, fits within the available vertical space, return success.
///
/// 2. If it does not fit, progressively reduce the scale factor by subtracting
///    `step_scale_factor` on each iteration:
///
///    a. After each reduction, check again whether the block fits at `current_top_y`.
///       If it fits, return success.
///
///    b. If the scale factor drops below `min_allowed_scale`, stop reducing and
///       proceed to step 3.
///
/// 3. Attempt to place the block at the top of a new page
///    (`A4_DEFAULT_MARGINS.top`):
///
///    a. Repeat the same progressive‑reduction loop described in step 2a.
///
///    b. If the block still cannot fit even at the minimum allowed scale,
///       return an error. It is up to the caller to decide how to handle this
///       failure.
///
/// # Arguments
///
/// * `current_top_y` — the y‑coordinate on the current page where the block
///   would be placed.
/// 
/// * `caption_height` — the height of the caption block (from a [`pango::Layout`]).
/// 
/// * `original_image_height` — the unscaled image height in pixels.
/// 
/// * `original_scale_factor` — the scale factor that fits the image within
///   [`a4_default_content_width()`].
/// 
/// * `step_scale_factor` — the amount by which the scale factor is reduced on
///   each iteration when attempting to make the block fit.
/// 
/// * `min_allowed_scale` — the minimum acceptable scale factor. If the scale
///   falls below this value, layout is considered impossible.
///
/// # Returns
///
/// * [`ImageBlockLayoutInfo`] — on success, containing the chosen scale factor,
///   whether a new page is required, and the effective top‑y position.
/// 
/// * `std::error::Error` — if the block cannot be fitted even after progressive
///   reduction and a page break.
///
fn step_scale_image(
    current_top_y: f64,
    caption_height: f64,
    original_image_height: f64,
    original_scale_factor: f64,
    step_scale_factor: f64,
    min_allowed_scale: f64,
) -> Result<ImageBlockLayoutInfo, Box<dyn std::error::Error>> 
{
    // There are only two possible y coordinates: the current y and a
    // new page A4_DEFAULT_MARGINS.top.
    let y_positions = [current_top_y, A4_DEFAULT_MARGINS.top];

    for (i, &top_y) in y_positions.iter().enumerate() {
        let mut scale = original_scale_factor;

        loop {
            let image_height = original_image_height * scale;
            // Don't need to account for IMAGE_BLOCK_SPACING: the main objective 
            // is to fit the image on the available space. IMAGE_BLOCK_SPACING is 
            // accounted for as part of the y-coordinate of anything that comes 
            // after this image block.
            let block_height = image_height + caption_height;

            if top_y + block_height <= a4_default_content_height() {
                return Ok(ImageBlockLayoutInfo::new(
                    scale,
                    i == 1, // new_page?
                    top_y,
                ));
            }

            // Try reducing scale.
            scale -= step_scale_factor;

            if scale < min_allowed_scale {
                break; // Try next y position.
            }
        }
    }

    Err("Image block cannot fit even after scaling and page break".into())
}

/// Attempt to lay out an image together with its caption (an “image block”).
///
/// The process works as follows:
///
/// * Compute the scale factor required to fit the image within
///   [`a4_default_content_width()`].
///
/// * Apply `reduction_factor` to obtain the initial final scale factor.
///   (The image is not scaled yet; this value is only used for layout calculations.)
///
/// * Then apply the following algorithm to lay out the image block:
///
/// 1. If the image scaled by the final scale factor, together with its caption,
///    fits in the remaining space on the current page, render the block and
///    return successfully.
///
/// 2. If the block does not fit, progressively reduce the final scale factor by
///    multiplying it with `step_scale_factor`.
///
///    a. After each reduction, if the block fits on the current page, render it
///       and return successfully.
///
///    b. If the scale factor becomes smaller than `min_allowed_scale`, proceed
///       to step 3.
///
/// 3. Attempt to render the image block on a new page:
///
///    a. Repeat the progressive‑reduction loop described in step 2a.
///
///    b. If the block still does not fit even on a fresh page, return an error.
///       It is up to the caller to decide how to handle this failure.
///
/// # Arguments
///
/// * `image_width` — the original width of the image.
/// 
/// * `image_height` — the original height of the image.
/// 
/// * `caption_layout` — the fully prepared caption text [`Layout`] associated 
///   with the image.
/// 
/// * `top_y` — the y‑coordinate at which to place the top of the image.
/// 
/// * `context` — the Cairo PDF [`Context`].
/// 
/// * `config` — configuration parameters such as the caption font,
///   `reduction_factor`, whether to horizontally center the image and caption,
///   `step_scale_factor`, and `min_allowed_scale`.
///
/// # Returns
///
/// * [`ImageBlockMeasuredInfo`] — on success, the image block scaling, 
///   pagination, positioning, and measuring information.
/// 
/// * [`std::error::Error`] — if the block cannot be fitted even after progressive
///   reduction and a page break.
/// 
pub fn measure_image_block(image_width: f64,
    image_height: f64,
    caption_layout: &Layout,
    top_y: f64, 
    config: &Config,
) -> Result<ImageBlockMeasuredInfo, Box<dyn std::error::Error>> {
    let reduction_factor = config.image_block().reduction_factor();
    let centre_aligned = config.image_block().centre_aligned();
    let step_scale_factor= config.image_block().step_scale_factor();
    let min_allowed_scale= config.image_block().min_allowed_scale();

    let caption_height = layout_block_height(caption_layout);

    let scale_factor: f64 = get_scaling_factor(image_width) * reduction_factor;

    let scaled_res = step_scale_image(top_y, 
        caption_height, image_height, 
        scale_factor, step_scale_factor, min_allowed_scale)?;

    let x: f64 = if centre_aligned {
        let width: f64 = image_width * scaled_res.scale_factor();
        ( (a4_default_content_width() - width) / 2.0 ) + A4_DEFAULT_MARGINS.left
    } else { A4_DEFAULT_MARGINS.left };

    let scaled_image_height: f64 = image_height * scaled_res.scale_factor();
    Ok(ImageBlockMeasuredInfo::new(scaled_res, 
        caption_height,
        scaled_image_height,
        config.layout().image_block_spacing(), x))
}

/// Attempt to render an image block, whose PNG has been loaded, and whose scaled factor 
/// has been calculated. The caption [`pango::Layout`] has already been fully prepared 
/// and ready to be rendered.
/// 
/// # Arguments
/// 
/// * `image_surface` — the decoded PNG ready for rendering.
/// 
/// * `caption_layout` — the fully prepared caption text [`Layout`] associated 
///   with the image.
/// 
/// * `measured_info` — [`ImageBlockMeasuredInfo`], the image block scaling, 
///   positioning, and measuring information. 
/// 
/// * `context` — the Cairo PDF [`Context`].
/// 
/// * `config` — configuration parameter [`crate::config::Config`]'s  
///   `image_block().centre_aligned()`: whether to horizontally center the image and 
///   its caption. Only meaningful when the scaled image is narrower than the page width.
///  
pub fn render_image_block(image_surface: &ImageSurface, 
    caption_layout: &Layout, 
    measured_info: &ImageBlockMeasuredInfo,
    context: &Context,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    // Reserve the entire context. Painting an image will alter some context information.
    context.save().expect("Failed to save cairo context");

    // Move to the top-left content area (unscaled)    
    context.translate(measured_info.x_coordinate(), measured_info.block_top_y());
    
    // Apply scale transformation
    context.scale(measured_info.scale_factor(), measured_info.scale_factor());

    // Draw the Image onto the PDF Surface:
    // Set the image surface as the source pattern for drawing
    // Draw the image at (0, 0) in scaled coordinates.
    context.set_source_surface(image_surface, 0.0, 0.0)?;

    // Paint the source surface onto the current target surface (the PDF surface).
    context.paint()?;

    // Restore the original context.
    context.restore().expect("Failed to restore cairo context");
    
    let caption_top_y =
        measured_info.block_top_y() + measured_info.image_height();

    context.move_to(A4_DEFAULT_MARGINS.left, caption_top_y);

    if config.image_block().centre_aligned() {
        center_layout_block(&context, &caption_layout, a4_default_content_width());
    }
    show_layout(&context, &caption_layout);

    Ok(()) 
}

// To run test for this module only: 
// 
//     * cargo test image_layout::tests
//
//     * cargo test image_layout::tests::test_step_scale_image_unscalable -- --exact [--nocapture]
//     * cargo test image_layout::tests::test_step_scale_image_current_page -- --exact [--nocapture]
//     * cargo test image_layout::tests::test_step_scale_image_new_page -- --exact [--nocapture]
//     * cargo test image_layout::tests::test_measure_image_block_unscalable -- --exact [--nocapture]
//     * cargo test image_layout::tests::test_measure_image_block_current_page -- --exact [--nocapture]
//     * cargo test image_layout::tests::test_measure_image_block_new_page -- --exact [--nocapture]
//
#[cfg(test)]
/// Note: the image used in the test is ./img/unscalable.png, although a physical image is 
/// not strictly requried for the tests.
/// 
/// The dimension of the image is: width 964px, height 1600px.
/// 
/// The caption is `Fractal generated using GIMP 2. Image width 964px, height 1600px.`, 
/// using { family = "Be Vietnam Pro", size = 12, weight = "normal", style = "italic" } font.
/// The [`Layout`]'s height of this caption is `40.48046875`: I wrote a script to pull this 
/// value out.
/// 
mod tests {
    use crate::text_layout::{
        a4_layout_width, 
        create_text_layout,
    };

    use cairo::{Context, PdfSurface};
    use super::*;
    use crate::page_geometry::A4;

    const IMAGE_WIDTH: f64 = 964.0;
    const IMAGE_HEIGHT: f64 = 1600.0;

    const CAPTION: &str = "Fractal generated using GIMP 2. Image width 964px, height 1600px.";
    const CAPTION_HEIGHT: f64 = 40.48046875;
        
    fn create_config(step_scale_factor: &str, min_allowed_scale: &str) -> Config {
        // There is a risk that this config_str will fail to load in the future when 
        // the configuration Rust code change.
        let config_str = 
            "[fonts]\n \
            headers = [\n
                { family = \"Be Vietnam Pro\", size = 20, weight = \"bold\", style = \"italic\" },\n \
                { family = \"Be Vietnam Pro\", size = 16, weight = \"bold\", style = \"normal\" },\n \
                { family = \"Be Vietnam Pro\", size = 14, weight = \"bold\", style = \"italic\" },\n \
                { family = \"Be Vietnam Pro\", size = 15, weight = \"bold\", style = \"italic\" },\n \
                { family = \"Be Vietnam Pro\", size = 14, weight = \"normal\", style = \"normal\" },\n \
                { family = \"Be Vietnam Pro\", size = 13, weight = \"bold\",   style = \"normal\" }\n \
            ]\n \
            paragraph = { family = \"Be Vietnam Pro\", size = 12, weight = \"normal\", style = \"normal\" }\n \
            caption = { family = \"Be Vietnam Pro\", size = 12, weight = \"normal\", style = \"italic\" }\n \
            page_number = { family = \"Be Vietnam Pro\", size = 10, weight = \"bold\", style = \"normal\" }\n \
            [layout]\n \
            image_block_spacing = 6.0\n \
            [image_block]\n \
            reduction_factor = 1.0\n \
            centre_aligned = true\n \
            step_scale_factor = {step_scale_factor}\n \
            min_allowed_scale = {min_allowed_scale}"
            .replace("{step_scale_factor}", step_scale_factor)
            .replace("{min_allowed_scale}", min_allowed_scale);

        let config: Config = toml::from_str(&config_str)
            .expect("Failed to load test config string");

        config
    }

    #[test]
    /// Using `./img/unscalable.png` documented at the top of the test mod.
    /// 
    fn test_step_scale_image_unscalable() {
        let current_top_y = A4_DEFAULT_MARGINS.top;
        let original_scale_factor = get_scaling_factor(IMAGE_WIDTH);
        let step_scale_factor = 0.0;
        // min_allowed_scale of 1.0 implies only accept the fit-page-width scaled image 
        // size. Note that `step_scale_image()` is not responsible for `reduction_factor` -- 
        // the `original_scale_factor` passed to it is already accounted for `reduction_factor`,
        // which is not applied in this test.
        let min_allowed_scale = 1.0;

        let res = step_scale_image(current_top_y, 
            CAPTION_HEIGHT, 
            IMAGE_HEIGHT, 
            original_scale_factor, 
            step_scale_factor, 
            min_allowed_scale);

        let err = res.expect_err("Expected failure for unscalable image");
        assert!(err.to_string().contains("cannot fit"), "Unexpected error message");
    }

    #[test]
    /// Using `./img/unscalable.png` documented at the top of the test mod.
    /// 
    fn test_step_scale_image_current_page() {
        let current_top_y = A4_DEFAULT_MARGINS.top;
        let original_scale_factor = get_scaling_factor(IMAGE_WIDTH);
        // Ensure the image is scaled down enough to fit the page.
        let step_scale_factor = 0.1;
        let min_allowed_scale = 0.2;

        let res = step_scale_image(current_top_y, 
            CAPTION_HEIGHT, 
            IMAGE_HEIGHT, 
            original_scale_factor, 
            step_scale_factor, 
            min_allowed_scale);

        assert!(res.is_ok(), "Expected success for scalable image");
        let image_block = res.unwrap();

        assert_eq!(image_block.block_top_y(), A4_DEFAULT_MARGINS.top, "top y");
        assert_eq!(image_block.new_page(), false, "current page");
        assert!(image_block.scale_factor() <= original_scale_factor, "step vs original scale factor");
        assert!(image_block.scale_factor() >= min_allowed_scale, "step scale factor vs min allowed scale");

        let scaled_height = IMAGE_HEIGHT * image_block.scale_factor();
        assert!(image_block.block_top_y() + scaled_height + CAPTION_HEIGHT <= a4_default_content_height());
    }

    #[test]
    fn test_step_scale_image_new_page() {
        // At the 600.00 y-coordinate, the image block should be on a new page.
        let current_top_y = 600.0;
        let original_scale_factor = get_scaling_factor(IMAGE_WIDTH);
        // Ensure the image is scaled down enough to fit the page.
        let step_scale_factor = 0.1;
        let min_allowed_scale = 0.2;

        let res = step_scale_image(current_top_y, 
            CAPTION_HEIGHT, 
            IMAGE_HEIGHT, 
            original_scale_factor, 
            step_scale_factor, 
            min_allowed_scale);

        assert!(res.is_ok(), "Expected success for scalable image");
        let image_block = res.unwrap();

        assert_eq!(image_block.block_top_y(), A4_DEFAULT_MARGINS.top, "top y");
        assert_eq!(image_block.new_page(), true, "new page");
        assert!(image_block.scale_factor() <= original_scale_factor, "step vs original scale factor");
        assert!(image_block.scale_factor() >= min_allowed_scale, "step scale factor vs min allowed scale");

        let scaled_height = IMAGE_HEIGHT * image_block.scale_factor();
        assert!(image_block.block_top_y() + scaled_height + CAPTION_HEIGHT <= a4_default_content_height());
    }

    #[test]
    /// Using `./img/unscalable.png` documented at the top of the test mod.
    ///     
    /// A4 default width: 595.22 - 57.0 - 57.0 = 481.22
    /// A4 default height: 842.0 - 57.0 - 57.0 = 728.00
    /// 
    /// ./img/unscalable.png: 
    ///     Width: 964px, double 481.22
    ///     Height: 1600px, more than double 728.00
    ///
    /// At factors of 1.0 -- the height of the image can never be scaled to fit 728.00.
    fn test_measure_image_block_unscalable() {
        let config = create_config("0.0", "1.0");

        let pdf_file_name = "test_measure_image_block_unscalable.pdf";
        let surface = PdfSurface::new(A4.width, A4.height, pdf_file_name)
            .expect("Failed to create PDF surface");

        let context = Context::new(&surface)
            .expect("Failed to create context");
        
        let layout = create_text_layout(a4_layout_width(), CAPTION, 
            config.fonts().caption(), true, &context);

        let res= measure_image_block(IMAGE_WIDTH, 
            IMAGE_HEIGHT, &layout, A4_DEFAULT_MARGINS.top, &config);

        let err = res.expect_err("Expected failure for unscalable image");
        assert!(err.to_string().contains("cannot fit"), "Unexpected error message");
    }

    #[test]
    /// Using `./img/unscalable.png` documented at the top of the test mod.
    ///     
    /// A4 default width: 595.22 - 57.0 - 57.0 = 481.22
    /// A4 default height: 842.0 - 57.0 - 57.0 = 728.00
    /// 
    /// ./img/unscalable.png: 
    ///     Width: 964px, double 481.22
    ///     Height: 1600px, more than double 728.00
    ///
    /// `step_scale_factor = 0.1` and `min_allowed_scale = 0.2` ensures the image 
    /// fits into the page.
    fn test_measure_image_block_current_page() {
        let config = create_config("0.1", "0.2");

        let pdf_file_name = "test_measure_image_block_current_page.pdf";
        let surface = PdfSurface::new(A4.width, A4.height, pdf_file_name)
            .expect("Failed to create PDF surface");

        let context = Context::new(&surface)
            .expect("Failed to create context");

        let layout = create_text_layout(a4_layout_width(), CAPTION, 
            config.fonts().caption(), true, &context);

        let res= measure_image_block(IMAGE_WIDTH, 
            IMAGE_HEIGHT, &layout, A4_DEFAULT_MARGINS.top, &config);

        assert!(res.is_ok(), "Expected success for scalable image");

        let measured_info = res.unwrap();

        assert_eq!(measured_info.new_page(), false, "not new page");
        assert_eq!(measured_info.block_top_y(), A4_DEFAULT_MARGINS.top, "top y");
    }

    #[test]
    /// Using `./img/unscalable.png` documented at the top of the test mod.
    ///     
    /// A4 default width: 595.22 - 57.0 - 57.0 = 481.22
    /// A4 default height: 842.0 - 57.0 - 57.0 = 728.00
    /// 
    /// ./img/unscalable.png: 
    ///     Width: 964px, double 481.22
    ///     Height: 1600px, more than double 728.00
    ///
    /// `step_scale_factor = 0.1` and `min_allowed_scale = 0.2` ensures the image 
    /// fits into the page.
    /// 
    /// At y-coordinate of 600.00, the image block will be forced onto a new page.
    fn test_measure_image_block_new_page() {
        const TOP_Y: f64 = 600.00;

        let config = create_config("0.1", "0.2");

        let pdf_file_name = "test_measure_image_block_current_page.pdf";
        let surface = PdfSurface::new(A4.width, A4.height, pdf_file_name)
            .expect("Failed to create PDF surface");

        let context = Context::new(&surface)
            .expect("Failed to create context");

        let layout = create_text_layout(a4_layout_width(), CAPTION, 
            config.fonts().caption(), true, &context);

        let res= measure_image_block(IMAGE_WIDTH, 
            IMAGE_HEIGHT, &layout, TOP_Y, &config);

        assert!(res.is_ok(), "Expected success for scalable image");

        let measured_info = res.unwrap();

        assert!(measured_info.new_page(), "new page");
        assert_eq!(measured_info.block_top_y(), A4_DEFAULT_MARGINS.top, "top y");
    }

}