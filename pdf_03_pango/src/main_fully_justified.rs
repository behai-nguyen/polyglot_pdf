// 11/12/2025
// True justification, the left and right margins are flush.

use cairo::{PdfSurface, Context};
use cairo::glib::translate::ToGlibPtr;
use pango::{Layout, FontDescription, WrapMode};
use pango_sys::pango_layout_set_justify;
use pangocairo::functions::{create_layout, show_layout};

mod page_geometry;
use page_geometry::{
    A4,
    a4_default_content_width,
    A4_DEFAULT_MARGINS,
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

fn main() {
    let (font_description, pdf_file_name) = if cfg!(target_os = "windows") {
        ("Arial Unicode MS 12", "win_full_justified.pdf")
    } else {
        ("NotoSansTC-Regular 12", "ubuntu_full_justified.pdf")
    };
    
    let surface = PdfSurface::new(A4.width, A4.height, pdf_file_name).unwrap();
    let cr = Context::new(&surface).unwrap();

    let layout = create_layout(&cr);
    layout.set_text("Lịch sử Việt Nam từ năm 1945 đến nay, còn nhiều bí ẩn chưa được giải tỏa. Người bàng quan, các thế hệ sau, sẽ không thấy được những âm mưu thầm kín của ông Hồ đã tiêu diệt người quốc gia, nếu như chúng ta không phát hiện được những bí mật lịch sử đó. Chúng tôi may mắn được nhà sử học Chính Ðạo, tức tiến sĩ Vũ Ngự Chiêu, cho phép sử dụng nhiều tài liệu quý giá mà ông sao lục từ các văn khố, thư viện của bộ Thuộc Ðịa, bộ Ngoại Giao Pháp… để làm sáng tỏ nhiều uẩn khúc lịch sử, vốn bị cộng sản che giấu, nhiễu loạn từ hơn nửa thế kỷ qua. Chúng tôi chân thành cảm tạ tiến sĩ Chiêu. Trong loạt bài nầy, chúng tôi sẽ trưng bằng chứng về những hành vi phản bội quyền lợi dân tộc của ông Hồ. Nổi thao thức của ông Hồ lúc nầy là Việt Minh phải mắm chính quyền, không chia xẻ, nhượng bộ cho bất cứ đảng phái nào. Ðó là đường lối nhất quán, trước sau như một của đảng cộng sản. Ðây cũng là dự mưu, từ khi ngoài rừng núi Tân Trào kéo về Hà Nội. “Căn cứ vào kết quả của cuộc thảo luận của ông Hồ cùng các cán bộ, thấy rằng công cuộc phát triển cách mạng của họ sẽ dẫn đến 2 trường hợp:");
    layout.set_width((a4_default_content_width() * pango::SCALE as f64) as i32);
    
    // layout.set_justify(true);
    unsafe {
        pango_layout_set_justify(layout.to_glib_none().0, 1); // 1 = TRUE
    }

    layout.set_wrap(WrapMode::WordChar);

    let desc = FontDescription::from_string(font_description);
    layout.set_font_description(Some(&desc));
    
    cr.move_to(A4_DEFAULT_MARGINS.left, A4_DEFAULT_MARGINS.top);
    show_layout(&cr, &layout);

    surface.finish();

    println!("PDF written to: {pdf_file_name}");
}