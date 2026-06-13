/* 03/06/2026. */

//! Types describe the structure of a document.
//! The home of layout‑agnostic block metadata.

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
    /// [`crate::config::Config::block_spacing()`].image().after().
    spacing_after: f64,
    /// The x-coordinate of the image.
    x_coordinate: f64,
}

#[allow(dead_code)]
impl ImageBlockMeasuredInfo {
    pub fn new(layout_info: ImageBlockLayoutInfo,
        caption_height: f64,
        image_height: f64,
        spacing_after: f64,
        x_coordinate: f64,        
    ) -> Self {
        Self { layout_info, caption_height, image_height, 
            spacing_after, x_coordinate }
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

    pub fn spacing_after(&self) -> f64 {
        self.spacing_after
    }

    pub fn x_coordinate(&self) -> f64 {
        self.x_coordinate
    }

    pub fn block_height(&self) -> f64 {
        self.image_height + self.caption_height + self.spacing_after
    }
}

/// Physical layout fragments.
#[derive(Debug)]
pub enum PositionedBlock {
    /// `Header` and `Paragraph` have effectively been normalised to 
    /// be identical at this point, their layout configuration is 
    /// different, and hence the need to distinguish between the two.
    Header {
        /// Index to the original semantic [`bh_cmark::ast::AstBlock`].
        block_index: usize,
        /// Which page this fragment belongs to.
        page: usize,
        /// Where on the page it starts.
        y_offset: f64,
        /// First line of this fragment (layout header can break 
        /// into multiple lines).
        line_start: usize,
        /// Last line of this fragment (exclusive)
        line_end: usize,
    },
    Paragraph {
        /// Index to the original semantic [`bh_cmark::ast::AstBlock`].
        block_index: usize,
        /// Which page this fragment belongs to.
        page: usize,
        /// Where on the page it starts.
        y_offset: f64,
        /// First line of this fragment.
        line_start: usize,
        /// Last line of this fragment (exclusive)
        line_end: usize,
    },
    Image {
        /// Index to the original semantic [`bh_cmark::ast::AstBlock`].
        block_index: usize,
        /// Which page this fragment belongs to.
        page: usize,
        /// Image block pagination and rendering information.
        measured_info: ImageBlockMeasuredInfo,
    },
    Thematic {
        /// Index to the original semantic [`bh_cmark::ast::AstBlock`].
        block_index: usize,
        /// Which page this fragment belongs to.
        page: usize,
        /// Where on the page it starts.
        y_offset: f64,
    }
}

impl PositionedBlock {
    pub fn header(block_index: usize, 
        page: usize,
        y_offset: f64,
        line_start: usize,
        line_end: usize,
    ) -> Self {
        PositionedBlock::Header { 
            block_index,
            page,
            y_offset,
            line_start, 
            line_end
        }
    }

    pub fn paragraph(block_index: usize, 
        page: usize,
        y_offset: f64,
        line_start: usize,
        line_end: usize,
    ) -> Self {
        PositionedBlock::Paragraph { 
            block_index,
            page,
            y_offset,
            line_start, 
            line_end
        }
    }

    pub fn image(block_index: usize, 
        page: usize, 
        measured_info: ImageBlockMeasuredInfo
    ) -> Self {
        PositionedBlock::Image { block_index, page, measured_info }
    }

    pub fn thematic(block_index: usize,
        page: usize,
        y_offset: f64
    ) -> Self {
        PositionedBlock::Thematic { block_index, page, y_offset }
    }

    /// Index to the original semantic block.
    pub fn block_index(&self) -> usize {
        match self {
            PositionedBlock::Header { block_index, .. } | 
            PositionedBlock::Paragraph { block_index, .. } | 
            PositionedBlock::Image { block_index, .. } | 
            PositionedBlock::Thematic { block_index, .. } => *block_index,
        }
    }

    /// Which page this fragment belongs to.
    pub fn page(&self) -> usize {
        match self {
            PositionedBlock::Header { page, .. } | 
            PositionedBlock::Paragraph { page, .. } | 
            PositionedBlock::Image { page, .. } | 
            PositionedBlock::Thematic { page, .. } => *page,
        }
    }
}