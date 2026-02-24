/* 25/01/2026 */

//! Types describe the structure of a document.

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
}

/// Physical layout fragments.
#[derive(Debug)]
pub struct PositionedBlock {
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
}

pub const MAX_HEADER_LEVEL: usize = 6;

impl PositionedBlock {
    pub fn new(block_index: usize, 
        page: usize,
        y_offset: f64,
        line_start: usize,
        line_end: usize,
    ) -> Self {
        PositionedBlock {
            block_index,
            page,
            y_offset,
            line_start, 
            line_end
        }
    }

    /// Index to the original semantic block.
    pub fn block_index(&self) -> usize {
        self.block_index
    }

    /// Which page this fragment belongs to.
    pub fn page(&self) -> usize {
        self.page
    }

    /// Where on the page it starts.
    pub fn y_offset(&self) -> f64 {
        self.y_offset
    }

    /// First line of this fragment (for paragraphs).
    pub fn line_start(&self) -> usize {
        self.line_start
    }

    /// Last line of this fragment (exclusive)
    pub fn line_end(&self) -> usize {
        self.line_end
    }
}
