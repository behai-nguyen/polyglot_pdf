/* 03/06/2026. */

use std::{fs, process};

use bh_cmark::{
	ast::{AstBlock, InlineContent}, parser::parser::Parser, scanner::Scanner
};

use cairo::{Context, PdfSurface, ImageSurface};
use pango::{Layout, WrapMode};
use pangocairo::functions::*;

mod page_geometry;
use page_geometry::{
    A4,
    A4_DEFAULT_MARGINS,
    a4_default_content_width,
    a4_default_content_height,
};

mod config;
use config::{
    load_config,
    Config,
    FontSpec,
    FontConfig,
};

mod document;
use document::PositionedBlock;

mod font_utils;
use font_utils::create_font_attrs;

mod text_layout;
use crate::text_layout::a4_layout_width;

mod image_layout;
use image_layout::{measure_image_block, render_image_block};

/// `pango::Layout` computation caching:
///     - the shaped Pango layout
///     - the line count
///     - the line heights
///     - the block’s text and spans
/// 
/// Reuse it for both measurement and rendering.
#[allow(dead_code)]
#[derive(Debug)]
enum PreparedBlock {
    Header {
        /// Index to the original semantic `AstBlock`.
        block_index: usize,
        /// Header level.
        level: u8,
        /// The cached `pango::Layout`.
        layout: Layout,
        /// The computed line heights for each line within `layout`.
        line_heights: Vec<f64>,
    },
    Paragraph {
        /// Index to the original semantic `AstBlock`.
        block_index: usize,
        /// The cached `pango::Layout`.
        layout: Layout,
        /// The computed line heights for each line within `layout`.
        line_heights: Vec<f64>,
    },
    Image {
        /// Index to the original semantic `AstBlock`.
        block_index: usize,
        /// Actual caption text can be blank: treated as a non-blank string.
        caption_layout: Layout,
        /// The actual decoded PNG.
        image_surface: ImageSurface,
    },
    Thematic {
        /// Index to the original semantic `AstBlock`.
        block_index: usize,
        /// Horizontal line. Vertical space requirement is almost a constant:
        ///     `padding_top + stroke_width + padding_bottom`.
        block_height: f64,
    },
}

fn page_number(context: &Context, 
    page_no: usize,
    total_pages: usize,
    page_width: f64,
    page_height: f64,
    font_config: &FontConfig
) {
    // Draw page number centered at bottom
    let footer_layout = create_layout(context);

    footer_layout.set_text(&format!("{} of {}", page_no, total_pages));
    footer_layout.set_font_description(Some(&font_config.page_number().to_pango_description()));

    // Measure width of the page number
    let (ink, _) = footer_layout.extents();
    let text_width = ink.width() as f64 / pango::SCALE as f64;

    let x = ((page_width - text_width) / 2.0) + A4_DEFAULT_MARGINS.left;
    let y = page_height - A4_DEFAULT_MARGINS.bottom;

    context.move_to(x, y);
    show_layout(&context, &footer_layout);
}

/// Read the Markdown text file, parse and turn each line into [`Block`] enum 
/// representations.
/// 
/// # Arguments
/// 
/// * `file_name` — the Markdown text file name.
/// 
/// # Returns
/// 
/// * [`Vec<Block>`] — the [`Block`] enum representations for each text line in 
///   the Markdown text file.
/// 
/// * [`std::error::Error`] — if some error occurs during file opening and reading.
/// 
fn parse_blocks_from_file(file_name: &str) -> Result<Vec<AstBlock>, Box<dyn std::error::Error>> {
    // Read input text file.
    let text = fs::read_to_string(file_name)?;

	let mut scanner = Scanner::new(&text);
    let tokens = match scanner.scan_tokens() {
		Ok(tokens) => tokens,
		Err(err) => {
			return Err(err.into());
		}
	};

    let mut parser = Parser::new(&tokens);
    let parse_output = parser.parse();
	
	if parse_output.has_error() {
		return Err(parse_output.errors().join("\n").into())
	}

    Ok(parse_output.into_blocks())
}

fn block_font<'a>(block: &'a AstBlock, font_config: &'a FontConfig) -> &'a FontSpec {
    match block {
        AstBlock::Header {level, content: _} => { 
            &font_config.headers()[*level as usize - 1]
        }
        AstBlock::Paragraph { content: _ } => {
            font_config.paragraph()
        }
        AstBlock::Image { path: _, alt: _ } => font_config.caption(),
        _ => font_config.paragraph(),
    }
}

fn measure_line_height(line_index: i32, layout: &Layout) -> f64 {
    if let Some(line) = layout.line(line_index) {
        let (_ink, logical) = line.extents();
        logical.height() as f64 / pango::SCALE as f64
    } else {
        panic!("measure_line_height: layout.line({line_index}) returned None");

        // eprintln!("Warning: layout.line({line_index}) returned None");
        // 0.0
    }
}

/// Text layout for each [`AstBlock`] enum.
fn create_layout_for_block(block: &AstBlock, 
    font_config: &FontConfig, 
    context: &Context
) -> Layout {
    let layout: Layout = create_layout(context);
    
    // Set width, wrap, justify
    layout.set_width(a4_layout_width());
    layout.set_wrap(WrapMode::WordChar);
    layout.set_justify(true);
    
    let font_spec = block_font(block, font_config);
    layout.set_font_description(Some(&font_spec.to_pango_description()));

    let display_text = |inline_content: &InlineContent| {
        let attrs = pango::AttrList::new();
        for span in inline_content.spans() {
            for attr in create_font_attrs(span) {
                attrs.insert(attr);
            }
        }

        layout.set_attributes(Some(&attrs));
        layout.set_text(inline_content.text());
    };

    match block {
        AstBlock::Header { level: _, content } => display_text(content),
        AstBlock::Paragraph { content } => display_text(content),
        AstBlock::Image { path: _, alt } => 
            display_text(alt),
        _ => {}
    }    

    layout
}

/// Convert semantic [`AstBlock`]s into their [`PreparedBlock`] equivalents.
fn prepare_blocks(
    blocks: &[AstBlock],
    config: &Config,
    context: &Context
) -> Vec<PreparedBlock> {
    let mut prepared = Vec::new();

    for (i, block) in blocks.iter().enumerate() {
        let layout = create_layout_for_block(block, config.fonts(), context);

        match block {
            AstBlock::Header { level, content: _ } => {
                let line_heights = (0..layout.line_count())
                    .map(|i| measure_line_height(i, &layout))
                    .collect();

                prepared.push(PreparedBlock::Header {
                    block_index: i,
                    level: *level,
                    layout,
                    line_heights,
                });
            },
            AstBlock::Image { path, alt: _ } => {
                let mut img_file = fs::File::open(path)
                    .unwrap_or_else(|_| panic!("Failed to open image PNG file: {}", path));

                let image_surface = ImageSurface::create_from_png(&mut img_file)
                    .map_err(|e| format!("Failed to create image surface from PNG: {}", e))
                    .expect("Failed to decode PNG image");

                prepared.push(PreparedBlock::Image {
                    block_index: i,
                    caption_layout: layout, 
                    image_surface: image_surface,
                });
            },
            AstBlock::Thematic => {
                prepared.push(PreparedBlock::Thematic {
                    block_index: i, 
                    block_height: config.block_spacing().thematic().before() +
                        config.horizontal_break().stroke_width() + 
                        config.block_spacing().thematic().after()
                });
            },
            _ => {
                let line_heights = (0..layout.line_count())
                    .map(|i| measure_line_height(i, &layout))
                    .collect();

                prepared.push(PreparedBlock::Paragraph {
                    block_index: i,
                    layout,
                    line_heights,
                });
            }
        }
    }

    prepared
}

fn header(pos_blocks: &mut Vec<PositionedBlock>, 
    block_index: usize, 
    current_page: usize, 
    y_offset: f64, 
    start_line: usize, 
    line_index: usize
) {
    pos_blocks.push(PositionedBlock::header(block_index, current_page, 
        y_offset, start_line, line_index));
}

fn paragraph(pos_blocks: &mut Vec<PositionedBlock>, 
    block_index: usize, 
    current_page: usize, 
    y_offset: f64, 
    start_line: usize, 
    line_index: usize
) {
    pos_blocks.push(PositionedBlock::paragraph(block_index, current_page, 
        y_offset, start_line, line_index));
}

/// At this point, both [`AstBlock::Header`] and [`AstBlock::Paragraph`] have 
/// been normalised into [`Layout`] and lines within [`Layout`].
/// 
/// The only difference between these two is [`AstBlock::Header`] has `level`.
fn text_block(line_heights: &[f64], 
    pos_blocks: &mut Vec<PositionedBlock>, 
    block_index: usize, 
    level: u8, 
    current_page: &mut usize, 
    y: &mut f64, 
    y_offset: &mut f64, 
    layout: &Layout,
    spacing_before: f64, 
    spacing_after: f64
) {
    // Start of a new block.
    *y += spacing_before;
    *y_offset = *y;

    let mut start_line: usize = 0;        
    for (line_index, line_height) in line_heights.iter().enumerate() {
        if *y + line_height > a4_default_content_height() {
            // This AstBlock spans multiple PositionedBlocks.
            if level > 0 {
                header(pos_blocks, block_index, *current_page, 
                    *y_offset, start_line, line_index);
            } else {
                paragraph(pos_blocks, block_index, *current_page, *y_offset, 
                    start_line, line_index);
            }

            start_line = line_index as usize;
            *current_page += 1; 
            *y = A4_DEFAULT_MARGINS.top;
            *y_offset = *y; 
            // Advance y so the next line does not overlap.
            // `line_height` of the line that `line_index` points to.
            *y += line_height;
        } else {
            *y += line_height;
        }
    }

    if level > 0 {
        header(pos_blocks, block_index, *current_page, *y_offset, 
            start_line, layout.line_count() as usize);
    } else {
        paragraph(pos_blocks, block_index, *current_page, *y_offset, 
            start_line, layout.line_count() as usize);
    }

    // Next Block
    *y += spacing_after;
    *y_offset = *y;
}

/// Preparing [`PositionedBlock`] vector for pagination and rendering.
/// The number of elements in this vector can be more than in the 
/// [`AstBlock`] vector and [`PreparedBlock`] vector.
fn measure_block(prepared_blocks: &[PreparedBlock], 
    config: &Config
) -> Result<Vec<PositionedBlock>, Box<dyn std::error::Error>> {
    let mut current_page = 1;
    let mut y = A4_DEFAULT_MARGINS.top;
    let mut y_offset = A4_DEFAULT_MARGINS.top;

    let mut pos_blocks: Vec<PositionedBlock> = Vec::new();

    for block in prepared_blocks {
        match block {
            PreparedBlock::Header { block_index, level, layout, line_heights } => {
                let block_spacing = config.block_spacing().heading();

                text_block(line_heights, &mut pos_blocks, *block_index, *level, 
                    &mut current_page, &mut y, &mut y_offset, layout,
                    block_spacing.before(*level), block_spacing.after(*level));
            },
            PreparedBlock::Paragraph { block_index, layout, line_heights } => {
                let block_spacing = config.block_spacing().paragraph();

                text_block(line_heights, &mut pos_blocks, *block_index, 0, 
                    &mut current_page, &mut y, &mut y_offset, layout, 
                    block_spacing.before(), block_spacing.after());
            },
            PreparedBlock::Image { block_index, caption_layout, image_surface } => {
                let measured_info = measure_image_block(
                    image_surface.width() as f64, image_surface.height() as f64, 
                    caption_layout, y_offset, config
                )?;

                // Work out the page for the image block.
                let page_for_block = if measured_info.new_page() {
                    current_page + 1
                } else {
                    current_page
                };

                // Remember the page placement for the image block.
                pos_blocks.push(
                    PositionedBlock::image(*block_index, page_for_block, measured_info)
                );

                // The code below matches the text‑block logic more closely.
                // This ensures:
                // 
                // - the block is placed on the correct page.
                // - `y` is correct for the next block.
                // - `y_offset` is correct.
                // - pagination state matches measurement state.
                if measured_info.new_page() {
                    current_page += 1;
                    y = A4_DEFAULT_MARGINS.top + measured_info.block_height();
                    y_offset = A4_DEFAULT_MARGINS.top;
                } else {
                    y = measured_info.block_top_y() + measured_info.block_height();
                    y_offset = measured_info.block_top_y();
                }

                // Guardrail when pagination logic is out of sync with measurement logic.
                debug_assert!((measured_info.block_top_y() - y_offset).abs() < 0.1);
                debug_assert!((y - (measured_info.block_top_y() + measured_info.block_height())).abs() < 0.1);
            },
            PreparedBlock::Thematic { block_index, block_height } => {
                // The horizontal line are to be drawn on a new page: 
                //     TO_DO: not desirable.
                if y + *block_height > a4_default_content_height() {
                    current_page += 1; 
                    y = A4_DEFAULT_MARGINS.top;
                    y_offset = y
                }

                pos_blocks.push( 
                    PositionedBlock::thematic(*block_index, current_page, 
                        y_offset + config.block_spacing().thematic().before())
                );

                y += *block_height;
            }
        }

    }

    Ok(pos_blocks)
}

/// Write all [`PositionedBlock`]s to PDF using the available pagination info.
/// 
/// All layout information has already been calculated by [`measure_block()`],
/// this function 
fn output_positioned_block(context: &Context,
    config: &Config,
    prepared: &PreparedBlock,
    pos_block: &PositionedBlock
) {
    let text = |y_offset: f64, 
        line_start: usize, line_end: usize, layout: &Layout, 
        line_heights: &[f64]| {

        let mut y = y_offset;
        for i in line_start..line_end {
            if let Some(line) = layout.line(i as i32) {
                context.move_to(A4_DEFAULT_MARGINS.left, y);
                show_layout_line(context, &line);

                // Use the precomputed line height.
                y += line_heights[i];
            }
        }        
    };

    match (pos_block, prepared) {
        (PositionedBlock::Header { y_offset, line_start, line_end, .. },
        PreparedBlock::Header { layout, line_heights, .. }) => {
            text(*y_offset, *line_start, *line_end, layout, line_heights);
        },
        (PositionedBlock::Paragraph { y_offset, line_start, line_end, .. },
        PreparedBlock::Paragraph { layout, line_heights, .. }) => {
            text(*y_offset, *line_start, *line_end, layout, line_heights);
        },
        (PositionedBlock::Image { measured_info, .. },
        PreparedBlock::Image { caption_layout, image_surface, .. }) => {
            let _ = render_image_block(image_surface, caption_layout, 
                measured_info, context, config);
        },
        (PositionedBlock::Thematic { y_offset, .. }, PreparedBlock::Thematic { .. }) => {
            context.save().expect("Failed to save Cairo context");

            context.move_to(A4_DEFAULT_MARGINS.left, *y_offset);
            context.line_to(A4.width - A4_DEFAULT_MARGINS.right, *y_offset);
            
            context.set_line_width(config.horizontal_break().stroke_width());
            
            context.set_source_rgb(config.horizontal_break().colour().r(), 
            config.horizontal_break().colour().g(), 
            config.horizontal_break().colour().b());
            let _ = context.stroke();
            
            // Restore the original context.
            context.restore().expect("Failed to restore Cairo context");
        },
        _ => debug_assert!(false, "Mismatched PreparedBlock and PositionedBlock variants"),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (config_file_name, pdf_file_name) = if cfg!(target_os = "windows") {
        ("./config/config.toml", "win_essay.pdf")
    } else {
        ("./config/config.toml", "ubuntu_essay.pdf")
    };

    let config = load_config(config_file_name)?;

    let blocks = match parse_blocks_from_file("./text/essay.txt") {
		Ok(blocks) => blocks,
		Err(err) => {
			println!("\nError: {}", err.to_string());
			process::exit(1);
		}
	};
	
    let surface = PdfSurface::new(A4.width, A4.height, pdf_file_name)?;

    let context = Context::new(&surface)?;

    let prepared_blocks = prepare_blocks(&blocks, &config, &context);

    let pos_blocks = measure_block(&prepared_blocks, &config)?;

    let total_pages: usize = pos_blocks[pos_blocks.len() - 1].page();
    let mut current_page: usize = 1;

    for pos_block in pos_blocks {
        if pos_block.page() != current_page {
            page_number(&context, current_page, total_pages, 
                a4_default_content_width(), A4.height, 
                &config.fonts());

            let _ = context.show_page();
            current_page = pos_block.page();
        };

        output_positioned_block(&context, &config, &prepared_blocks[pos_block.block_index()], 
            &pos_block);
    }

    page_number(&context, current_page, total_pages, 
        a4_default_content_width(), A4.height, 
        &config.fonts());

    surface.finish();

    println!("PDF written to: {pdf_file_name}");

	Ok(())
}