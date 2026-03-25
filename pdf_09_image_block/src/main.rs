/* 16/02/2026 */

//! Reads the Markdown `./text/essay.txt`, and using font information in 
//! `./config/config.toml`, and converts the Markdown text file to PDF.
//! 
//! PDF generation supports Markdown header 1 to 6, bold, italic, and 
//! bold italic format.

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
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
    FontSpec,
    FontConfig,
    Config,
    load_config
};

mod document;
use document::{
    Block,
    PositionedBlock,
    MAX_HEADER_LEVEL,
};

mod font_utils;
use font_utils::create_font_attrs;

mod image_block_parser;
use image_block_parser::parse_image_block;

mod text_layout;
use crate::text_layout::a4_layout_width;

mod image_layout;
use image_layout::{measure_image_block, render_image_block};

mod inline_parser;
use inline_parser::{parse_inline, InlineParseResult, reserve_asterisk};

/// `pango::Layout` computation caching:
///     - the shaped Pango layout
///     - the line count
///     - the line heights
///     - the block’s text and spans
/// 
/// Reuse it for both measurement and rendering.
#[allow(dead_code)]
enum PreparedBlock {
    Text {
        /// Index to the original semantic Block.
        block_index: usize,
        /// The cached `pango::Layout`.
        layout: Layout,
        /// The computed line heights for each line within `layout`.
        line_heights: Vec<f64>,
    },
    Image {
        block_index: usize,
        /// Actual caption text can be blank: treated as a non-blank string.
        caption_layout: Layout,
        /// The actual decoded PNG.
        image_surface: ImageSurface,
    }
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

fn detect_header(line: &str) -> Option<(usize, &str)> {
    let mut count = 0;
    for c in line.chars() {
        if c == '#' {
            count += 1;
        } else {
            break;
        }
    }

    if count > 0 && count <= MAX_HEADER_LEVEL {
        Some((count, line[count..].trim()))
    } else {
        None
    }
}

fn detect_image_block_text(line: &str) -> Option<&str> {
    let s = line.trim();
    if s.starts_with("![") && s.contains(']') && s.contains('(') {
        Some(s)
    } else {
        None
    }
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
fn parse_blocks_from_file(file_name: &str) -> Result<Vec<Block>, Box<dyn std::error::Error>> {
    let file_path = Path::new(file_name);
    let file = fs::File::open(file_path)?;

    let mut blocks: Vec<Block> = Vec::new();

    let reader = BufReader::new(file);
    for line_result in reader.lines() {
        let line = line_result?;

        if let Some(header) = detect_header(&line) {
            blocks.push(
                Block::Header { level: header.0 as u8, text: header.1.to_string() },
            )
        } else if let Some(image_block_text) = detect_image_block_text(&line) {
            match parse_image_block(image_block_text) {
                Ok(block_img_info) => {
                    let caption = (!block_img_info.caption().is_empty())
                        .then(|| block_img_info.caption().to_string());                         
                    blocks.push(
                        Block::Image { path: block_img_info.path().to_string(), 
                            caption: caption }
                    );
                },
                Err(_) => {
                    // Invalid image block text is treated as a `Block::Paragraph`.
                    let InlineParseResult { text, spans } = parse_inline(&line);
                    blocks.push(Block::Paragraph { text, spans });
                }
            }
        } else {
            let InlineParseResult { text, spans } = parse_inline(&line);
            blocks.push(Block::Paragraph { text, spans });
        }
    }

    Ok(blocks)
}

fn block_font<'a>(block: &'a Block, font_config: &'a FontConfig) -> &'a FontSpec {
    match block {
        Block::Header {level, text: _} => { 
            &font_config.headers()[*level as usize - 1]
        }
        Block::Paragraph {text: _, spans: _} => {
            font_config.paragraph()
        },
        Block::Image {path: _, caption: _} => {
            font_config.caption()
        }
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

/// Text layout for each [`Block`] enum.
fn create_layout_for_block(block: &Block, 
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

    match block {
        Block::Header {level: _, text} => {
            layout.set_text(text);
        }

        Block::Paragraph {text, spans} => {
            let attrs = pango::AttrList::new();
            for span in spans {
                for attr in create_font_attrs(span) {
                    attrs.insert(attr);
                }
            }

            layout.set_attributes(Some(&attrs));                
            layout.set_text(&reserve_asterisk(text));
        },

        Block::Image { path: _, caption} => {
            let caption_text = caption.as_deref().unwrap_or("");
            layout.set_text(caption_text);
        }
    }    

    layout
}

/// Convert semantic [`Block`]s into their [`PreparedBlock`] equivalents.
fn prepare_blocks(
    blocks: &[Block],
    font_config: &FontConfig,
    context: &Context
) -> Vec<PreparedBlock> {
    let mut prepared = Vec::new();

    for (i, block) in blocks.iter().enumerate() {
        let layout = create_layout_for_block(block, font_config, context);

        match block {
            Block::Image { path, caption: _ } => {
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
            _ => {
                let line_heights = (0..layout.line_count())
                    .map(|i| measure_line_height(i, &layout))
                    .collect();

                prepared.push(PreparedBlock::Text {
                    block_index: i,
                    layout,
                    line_heights,
                });
            }
        }
    }

    prepared
}

/// Preparing [`PositionedBlock`] vector for pagination and rendering.
/// The number of elements in this vector can be more than in the 
/// [`Block`] vector and [`PreparedBlock`] vector.
fn measure_block(prepared_blocks: &[PreparedBlock], 
    config: &Config
) -> Result<Vec<PositionedBlock>, Box<dyn std::error::Error>> {
    let mut current_page = 1;
    let mut y = A4_DEFAULT_MARGINS.top;
    let mut y_offset = A4_DEFAULT_MARGINS.top;

    let mut pos_blocks: Vec<PositionedBlock> = Vec::new();

    for block in prepared_blocks {
        match block {
            PreparedBlock::Text { block_index, layout, line_heights } => {
                let mut start_line: usize = 0;

                for (line_index, line_height) in line_heights.iter().enumerate() {

                    if y + line_height > a4_default_content_height() {
                        // This Block spans multiple PositionedBlocks.
                        pos_blocks.push( 
                            PositionedBlock::new_text(*block_index, current_page, y_offset, 
                                start_line, line_index as usize) );

                        start_line = line_index as usize;
                        current_page += 1; 
                        y = A4_DEFAULT_MARGINS.top;
                        y_offset = y; 
                        // Advance y so the next line does not overlap.
                        // `line_height` of the line that `line_index` points to.
                        y += line_height;
                    } else {
                        y += line_height;
                    }
                }

                pos_blocks.push( 
                    PositionedBlock::new_text(*block_index, current_page, y_offset, 
                        start_line, layout.line_count() as usize) 
                );

                // Next Block
                y_offset = y;
            }

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
                    PositionedBlock::new_image(*block_index, page_for_block, measured_info)
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
            }            
        }

    }

    Ok(pos_blocks)
}

/// Write all [`PositionedBlock`]s to PDF using the available pagination info.
fn output_positioned_block(context: &Context,
    config: &Config,
    prepared: &PreparedBlock,
    pos_block: &PositionedBlock
) {
    match (pos_block, prepared) {
        (PositionedBlock::Text { y_offset, line_start, line_end, .. },
        PreparedBlock::Text { layout, line_heights, .. }) => {
            let mut y = *y_offset;
            for i in *line_start..*line_end {
                if let Some(line) = layout.line(i as i32) {
                    context.move_to(A4_DEFAULT_MARGINS.left, y);
                    show_layout_line(context, &line);

                    // Use the precomputed line height.
                    y += line_heights[i];
                }
            }
        }

        (PositionedBlock::Image { measured_info, .. },
        PreparedBlock::Image { caption_layout, image_surface, .. }) => {
            let _ = render_image_block(image_surface, caption_layout, 
                measured_info, context, config);
        }

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

    let blocks = parse_blocks_from_file("./text/essay.txt")?;

    let surface = PdfSurface::new(A4.width, A4.height, pdf_file_name)?;

    let context = Context::new(&surface)?;

    let prepared_blocks = prepare_blocks(&blocks, 
        &config.fonts(), &context);

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