/* 25/01/2026 */

//! Types describe the structure of a document.
//! The home of layout‑agnostic block metadata.

/// Markdown supports six header levels.
pub const MAX_HEADER_LEVEL: usize = 6;

/// Applying both bold and italic on a byte-range produces bold italic text.
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum SpanStyle {
    Normal,
    Bold,
    Italic,
}

/// Individual slices within the paragraph text with different style.
/// `Span` can be adjacent or overlapped / nested.
/// 
/// The below markdown results in adjacent `Span`s:
///      "— **Tưởng Vĩnh Kính**, Hồ Chí Minh Tại *Trung Quốc*, Thượng Huyền dịch, \
///       ***trang 339***."
/// 
/// Which would produces the `Span`s:
///     [
///         Span { start: 0, end: 4, style: Normal }
///         Span { start: 4, end: 24, style: Bold }
///         Span { start: 24, end: 47, style: Normal }
///         Span { start: 47, end: 59, style: Italic }
///         Span { start: 59, end: 87, style: Normal }
///         Span { start: 87, end: 96, style: Bold }
///         Span { start: 87, end: 96, style: Italic }
///         Span { start: 96, end: 97, style: Normal }
///     ]
/// 
/// The following markdown results in overlapped / nested `Span`s:
///     "**Không đọc *sử* không đủ tư cách nói chuyện *chính trị*.**"
/// 
/// And it produces the `Span`s:
///     [
///         Span { start: 0, end: 69, style: Bold }, 
///         Span { start: 14, end: 18, style: Italic }, 
///         Span { start: 56, end: 68, style: Italic }
///     ]
#[derive(Debug, Clone)]
pub struct Span {
    /// The start byte of a text slice with a a specific style.
    start: usize,
    /// The end byte of a text slice with a a specific style.
    end: usize,
    /// The style of the text slice indexed by `start`..`slice`.
    style: SpanStyle,
}

impl Span {
    pub fn new(start: usize, 
        end: usize,
        marker_count: u8,
    ) -> Self {
        let style= match marker_count {
            1 => SpanStyle::Italic,
            2 => SpanStyle::Bold,
            _ => SpanStyle::Normal,
        };

        Span { start, end, style, }
    }

    pub fn normal(start: usize, end: usize) -> Self {
        Span { start, end, style: SpanStyle::Normal }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn style(&self) -> &SpanStyle {
        &self.style
    }
}    

/// Semantic document structure.
#[derive(Debug, Clone)]
pub enum Block {
    /// The text block / line is a header.
    Header { level: u8, text: String },
    /// `text`: the clean text block / paragraph / blank line is a normal text.
    /// `spans`: byte-ranges and their styles for slices in `text`.
    Paragraph { text: String, spans: Vec<Span> },
    /// Encapsulates `![caption](relative/path/to/image.png)`.
    /// `path`: `relative/path/to/image.png`.
    /// 'caption`: `caption`.
    Image { path: String, caption: Option<String>, }
}

/// The layout information for the current image and its caption.
#[derive(Debug, Clone, Copy)]
pub struct ImageBlockLayoutInfo {
    /// The final scale factor for the image.
    scale_factor: f64, 
    /// Is the image block on a new page.
    new_page: bool,
    /// The y-coordinate of the image block.
    block_top_y: f64,
}

#[allow(dead_code)]
impl ImageBlockLayoutInfo {
    pub fn new(scale_factor: f64, 
        new_page: bool, 
        block_top_y: f64
    ) -> Self {
        ImageBlockLayoutInfo { scale_factor, new_page, block_top_y }
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    pub fn new_page(&self) -> bool {
        self.new_page
    }

    pub fn block_top_y(&self) -> f64 {
        self.block_top_y
    }
}

/// The image block has been successfully scaled to fit, this struct encapsulates 
/// all information necessary for pagination and rendering of the image block.
#[derive(Debug, Clone, Copy)]
pub struct ImageBlockMeasuredInfo {
    layout_info: ImageBlockLayoutInfo,
    /// The layout ([`pango::Layout`]) height of the caption text.
    caption_height: f64,
    /// The effective height of the final scaled image.
    image_height: f64,
    /// [`crate::config::Config::layout()`]'s `image_block_spacing`.
    image_block_spacing: f64,
    /// The x-coordinate of the image.
    x_coordinate: f64,
}

#[allow(dead_code)]
impl ImageBlockMeasuredInfo {
    pub fn new(layout_info: ImageBlockLayoutInfo,
        caption_height: f64,
        image_height: f64,
        image_block_spacing: f64,
        x_coordinate: f64,        
    ) -> Self {
        Self { layout_info, caption_height, image_height, 
            image_block_spacing, x_coordinate }
    }

    pub fn layout_info(&self) -> &ImageBlockLayoutInfo {
        &self.layout_info
    }

    pub fn scale_factor(&self) -> f64 {
        self.layout_info.scale_factor
    }

    pub fn new_page(&self) -> bool {
        self.layout_info.new_page
    }

    pub fn block_top_y(&self) -> f64 {
        self.layout_info.block_top_y
    }

    pub fn caption_height(&self) -> f64 {
        self.caption_height
    }

    pub fn image_height(&self) -> f64 {
        self.image_height
    }

    pub fn image_block_spacing(&self) -> f64 {
        self.image_block_spacing
    }

    pub fn x_coordinate(&self) -> f64 {
        self.x_coordinate
    }

    pub fn block_height(&self) -> f64 {        
        self.image_height + self.caption_height + self.image_block_spacing
    }
}

/// Physical layout fragments.
#[derive(Debug)]
pub enum PositionedBlock {
    Text {
        /// Index to the original semantic Block.
        block_index: usize,
        /// Which page this fragment belongs to.
        page: usize,
        /// Where on the page it starts.
        y_offset: f64,
        /// First line of this fragment (for paragraphs).
        line_start: usize,
        /// Last line of this fragment (exclusive)
        line_end: usize,
    },
    Image {
        /// Index to the original semantic Block.
        block_index: usize,
        /// Which page this fragment belongs to.
        page: usize,
        /// Image block pagination and rendering information.
        measured_info: ImageBlockMeasuredInfo,
    }    
}

impl PositionedBlock {
    pub fn new_text(block_index: usize, 
        page: usize,
        y_offset: f64,
        line_start: usize,
        line_end: usize,
    ) -> Self {
        PositionedBlock::Text { 
            block_index,
            page,
            y_offset,
            line_start, 
            line_end
        }
    }

    pub fn new_image(block_index: usize, 
        page: usize, 
        measured_info: ImageBlockMeasuredInfo
    ) -> Self {
        PositionedBlock::Image { block_index, page, measured_info }
    }

    /// Index to the original semantic block.
    pub fn block_index(&self) -> usize {
        match self {
            PositionedBlock::Text { block_index, .. } | 
            PositionedBlock::Image { block_index, .. } => *block_index,
        }
    }

    /// Which page this fragment belongs to.
    pub fn page(&self) -> usize {
        match self {
            PositionedBlock::Text { page, .. } | 
            PositionedBlock::Image { page, .. } => *page,
        }
    }
}