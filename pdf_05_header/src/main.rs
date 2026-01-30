// 25/01/2026
//
// Implement header support for text to PDF: #, ..., ######.
//

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use cairo::{Context, PdfSurface};
use cairo::glib::translate::ToGlibPtr;
use pango::{Layout, FontDescription, WrapMode};
use pango_sys::pango_layout_set_justify;
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
    load_font_config
};

mod document;
use document::{
    Block,
    PositionedBlock,
    MAX_HEADER_LEVEL
};

pub trait LayoutExtJustify {
    fn set_justify(&self, justify: bool);
}

impl LayoutExtJustify for Layout {
    fn set_justify(&self, justify: bool) {
        unsafe {
            pango_layout_set_justify(self.to_glib_none().0, justify as i32);
        }
    }
}

impl FontSpec {
    pub fn to_pango_description(&self) -> FontDescription {
        let mut desc = FontDescription::new();

        desc.set_family(self.family());
        desc.set_size(self.size() * pango::SCALE);

        // Only takes effect if the font supports it.
        match self.weight() {
            "thin" => desc.set_weight(pango::Weight::Thin), 
            "light" => desc.set_weight(pango::Weight::Light), 
            "medium" => desc.set_weight(pango::Weight::Medium),            
            "bold" => desc.set_weight(pango::Weight::Bold),
            "normal" => desc.set_weight(pango::Weight::Normal),
            _ => desc.set_weight(pango::Weight::Normal),
        }

        // Only takes effect if the font supports it.
        match self.style() {
            "italic" => desc.set_style(pango::Style::Italic),
            "normal" => desc.set_style(pango::Style::Normal),
            _ => desc.set_style(pango::Style::Normal),
        }

        desc
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
        } else {
            blocks.push(Block::Paragraph {text: line})
        }
    }

    Ok(blocks)
}

fn block_font<'a>(block: &'a Block, font_config: &'a FontConfig) -> &'a FontSpec {
    match block {
        Block::Header {level, text: _} => { 
            &font_config.headers()[*level as usize - 1]
        }
        Block::Paragraph {text: _} => {
            font_config.paragraph()
        },
    }
}

fn block_text(block: &Block) -> &str {
    match block {
        Block::Header {level: _, text} => { text }
        Block::Paragraph {text: paragraph} => { paragraph },
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

fn measure_block(blocks: &[Block], 
    font_config: &FontConfig,
    context: &Context
) -> Result<Vec<PositionedBlock>, Box<dyn std::error::Error>> {

    let layout = create_layout(context);
    layout.set_width((a4_default_content_width() * pango::SCALE as f64) as i32);
    layout.set_justify(true);
    layout.set_wrap(WrapMode::WordChar);

    let mut current_page = 1;
    let mut y = A4_DEFAULT_MARGINS.top;
    let mut y_offset = A4_DEFAULT_MARGINS.top;
    let mut line_height: f64 = 0.0;

    let mut pos_blocks: Vec<PositionedBlock> = Vec::new();

    for (block_index, block) in blocks.iter().enumerate() {
        let font_spec = block_font(block, font_config);
        let text = block_text(block);

        let desc = font_spec.to_pango_description();
        layout.set_font_description(Some(&desc));
        layout.set_text(text);

        let mut start_line: usize = 0;
        for line_index in 0..layout.line_count() {
            line_height = measure_line_height(line_index, &layout);

            if y + line_height > a4_default_content_height() {
                // This Block spans multiple PositionedBlocks.
                pos_blocks.push( 
                    PositionedBlock::new(block_index, current_page, y_offset, line_height,
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
            PositionedBlock::new(block_index, current_page, y_offset, line_height,
                start_line, layout.line_count() as usize) );

        // Next Block
        y_offset = y;

    }

    Ok(pos_blocks)
}

fn output_positioned_block(layout: &Layout, 
    context: &Context, 
    font_config: &FontConfig, 
    pos_block: &PositionedBlock, 
    blocks: &[Block]) 
{
    let font_spec = block_font(&blocks[pos_block.block_index()], font_config);
    let layout_text = block_text(&blocks[pos_block.block_index()]);

    let desc = font_spec.to_pango_description();
    layout.set_font_description(Some(&desc));
    
    layout.set_text(&layout_text);

    let mut y: f64 = pos_block.y_offset();
    for i in pos_block.line_start()..pos_block.line_end() {
        if let Some(line) = layout.line(i as i32) {
            context.move_to(A4_DEFAULT_MARGINS.left, y);
            pangocairo::functions::show_layout_line(&context, &line);

            y += pos_block.line_height();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (config_file_name, pdf_file_name) = if cfg!(target_os = "windows") {
        ("./config/config-windows.toml", "win_essay.pdf")
    } else {
        ("./config/config-linux.toml", "ubuntu_essay.pdf")
    };

    let font_config = load_font_config(config_file_name)?;

    let blocks = parse_blocks_from_file("./text/essay.txt")?;

    let surface = PdfSurface::new(A4.width, A4.height, pdf_file_name)?;

    let context = Context::new(&surface)?;

    let pos_blocks = measure_block(&blocks, &font_config, &context)?;

    let layout = create_layout(&context);
    layout.set_width((a4_default_content_width() * pango::SCALE as f64) as i32);
    layout.set_justify(true);
    layout.set_wrap(WrapMode::WordChar);

    let total_pages: usize = pos_blocks[pos_blocks.len() - 1].page();
    let mut current_page: usize = 1;

    for pos_block in pos_blocks {
        if pos_block.page() != current_page {
            page_number(&context, current_page, total_pages, 
                a4_default_content_width(), A4.height, &font_config);

            let _ = context.show_page();
            current_page = pos_block.page();
        };

        output_positioned_block(&layout, &context, &font_config, &pos_block, &blocks);
    }

    page_number(&context, current_page, total_pages, 
        a4_default_content_width(), A4.height, &font_config);

    surface.finish();

    println!("PDF written to: {pdf_file_name}");
 
    Ok(())
}