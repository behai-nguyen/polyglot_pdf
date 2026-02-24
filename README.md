<!-- 29/10/2025 -->
# Polyglot PDF

Explorations in PDF generation using Rust, with support for multilingual text, font subsetting, and HarfBuzz integration.

## Related post(s)

1. [Rust FFI “Adventure” with the HarfBuzz Text Shaping Engine](https://behainguyen.wordpress.com/2025/10/28/rust-ffi-adventure-with-the-harfbuzz-text-shaping-engine/)

The code for this post is in [https://github.com/behai-nguyen/polyglot_pdf/tree/main/harfbuzz_ffi](https://github.com/behai-nguyen/polyglot_pdf/tree/main/harfbuzz_ffi).

Rust FFI, or <a href="https://doc.rust-lang.org/nomicon/ffi.html" title="Foreign Function Interface" target="_blank">Foreign Function Interface</a>, is a mechanism that allows Rust code to interact with programs written in other languages, such as C and C-compatible languages. The <a href="https://en.wikipedia.org/wiki/HarfBuzz" title="HarfBuzz" target="_blank">HarfBuzz</a> text shaping engine is written in C++.

In this article, we describe how to build the `HarfBuzz` text shaping engine on both Windows and Ubuntu. We then demonstrate how to write simple Rust code that calls the `hb_version_string()` function from `HarfBuzz` using FFI.

2. [Rust FFI Font Subsetting Using the HarfBuzz Text Shaping Engine](https://behainguyen.wordpress.com/2025/11/04/rust-ffi-font-subsetting-using-the-harfbuzz-text-shaping-engine/)

The code for this post is in [https://github.com/behai-nguyen/polyglot_pdf/tree/main/harfbuzz_font_subset](https://github.com/behai-nguyen/polyglot_pdf/tree/main/harfbuzz_font_subset).

Loosely speaking, **font subsetting** involves extracting only the characters we need from a font program, such as a TrueType `.ttf` file. The `Arial Unicode MS` font program is around 20MB. If we need only a few Vietnamese characters, we can extract and use those, resulting in a font subset of just a few kilobytes.

This article focuses on *font subsetting* on Windows and Ubuntu as a standalone process. We begin by installing a few standalone font tools on Windows, then explore the font subsetting workflow using the HarfBuzz library.

3. [Rust: Multilingual PDFs — an Introductory Study](https://behainguyen.wordpress.com/2025/11/11/rust-multilingual-pdfs-an-introductory-study/)

The code for this post is in [https://github.com/behai-nguyen/polyglot_pdf/blob/main/pdf_01](https://github.com/behai-nguyen/polyglot_pdf/blob/main/pdf_01).

A *“multilingual”* PDF simply means a **PDF document that supports displaying, copying, and pasting text in any Unicode-supported language**.

I’ve used several third-party libraries to generate multilingual PDF documents in previous jobs. I was under the impression that as long as we used the correct fonts, the text would render automatically—and so would copy and paste. But when I tried to create a multilingual PDF with Rust, I soon realised it’s not that simple!

With help from <a href="https://chatgpt.com/" title="ChatGPT" target="_blank">ChatGPT</a> and <a href="https://copilot.microsoft.com/" title="Copilot" target="_blank">Copilot</a>, we discovered that quite a bit of work is needed to get everything working. It’s not as simple as just embedding a font program in the PDF document.

In this article, we’ll take an introductory look at creating a multilingual PDF document that includes both Chinese and Vietnamese text. The Rust code we’re writing runs on both Windows and Ubuntu.

4. [Rust: PDFs — Basic Text Layout](https://behainguyen.wordpress.com/2025/12/07/rust-pdfs-basic-text-layout/)

The code for this post is in [https://github.com/behai-nguyen/polyglot_pdf/blob/main/pdf_02](https://github.com/behai-nguyen/polyglot_pdf/blob/main/pdf_02).

In the <a href="https://behainguyen.wordpress.com/2025/11/11/rust-multilingual-pdfs-an-introductory-study/" title="Rust: Multilingual PDFs — an Introductory Study" target="_blank">last article</a> we created a two-page PDF in which each page contained only a short Chinese and a Vietnamese sentence. In this article, we look at some basic text layout: how to fit a line of text within a given page width, and how many lines can fit within a given page height. We then create a simple PDF document with more than 70 pages of <strong>only</strong> Vietnamese text, using <strong>only</strong> a single font program and font size.

5. [Rust: PDFs — Build and Install Pango and Associated Libraries](https://behainguyen.wordpress.com/2025/12/19/rust-pdfs-build-and-install-pango-and-associated-libraries/)

At the conclusion of the <a href="https://behainguyen.wordpress.com/2025/12/07/rust-pdfs-basic-text-layout/#concluding-remarks" title="Rust: PDFs — Basic Text Layout" target="_blank">last article</a>, we mentioned that we would explore <a href="https://www.gtk.org/docs/architecture/pango" title="Pango Library" target="_blank"><code>Pango</code></a>, along with its associated libraries: <a href="https://github.com/fribidi/fribidi" title="GNU FriBidi" target="_blank"><code>GNU FriBidi</code></a> and <a href="https://www.cairographics.org/" title="CairoGraphics" target="_blank"><code>CairoGraphics</code></a>. In this article, we describe how to build and install them on both Ubuntu and Windows, and verify the installation using the native CLI tool included with <code>Pango</code>.

6. [Rust: PDFs — Exploring Layout with Pango and Cairo](https://behainguyen.wordpress.com/2025/12/27/rust-pdfs-exploring-layout-with-pango-and-cairo/)

The code for this post is in [https://github.com/behai-nguyen/polyglot_pdf/blob/main/pdf_03_pango](https://github.com/behai-nguyen/polyglot_pdf/blob/main/pdf_03_pango).

At the conclusion of the <a href="https://behainguyen.wordpress.com/2025/12/07/rust-pdfs-basic-text-layout/#concluding-remarks" title="Rust: PDFs — Basic Text Layout" target="_blank">fourth article</a>, we mentioned that we would explore text layout using <a href="https://www.gtk.org/docs/architecture/pango" title="Pango Library" target="_blank"><code>Pango</code></a> and its associated libraries: <a href="https://github.com/fribidi/fribidi" title="GNU FriBidi" target="_blank"><code>GNU FriBidi</code></a> and <a href="https://www.cairographics.org/" title="CairoGraphics" target="_blank"><code>CairoGraphics</code></a>. In the <a href="https://behainguyen.wordpress.com/2025/12/19/rust-pdfs-build-and-install-pango-and-associated-libraries/" title="Rust: PDFs — Build and Install Pango and Associated Libraries" target="_blank">fifth post</a>, we described how to build and install these libraries on both Ubuntu and Windows. In this article, we finally explore them in practice. The focus is on <strong>true justification</strong> — that is, both the left and right margins are flush, not ragged, much like the <a href="https://en.wikipedia.org/wiki/TeX" title="TeX typesetting system" target="_blank">TeX</a> typesetting system. We use the Vietnamese text file from the fourth article to generate a PDF of more than 150 pages.

7. [Rust: PDFs — Text Rotation with Cairo and Pango](https://behainguyen.wordpress.com/2026/01/16/rust-pdfs-text-rotation-with-cairo-and-pango/)

The code for this post is in [https://github.com/behai-nguyen/polyglot_pdf/blob/main/pdf_04_text_rotation](https://github.com/behai-nguyen/polyglot_pdf/blob/main/pdf_04_text_rotation).

My fascination with <a href="https://www.gtk.org/docs/architecture/pango" title="Pango Library" target="_blank"><code>Pango</code></a> and <a href="https://www.cairographics.org/" title="CairoGraphics" target="_blank"><code>CairoGraphics</code></a> has led me to explore text rotation. I find it very interesting. It becomes straightforward once we understand a few key ideas. In this article, we focus on ±90° rotation for left‑to‑right text only.

8. [Rust: PDFs — Pango and Cairo Layout — Supporting Headers](https://behainguyen.wordpress.com/2026/01/30/rust-pdfs-pango-and-cairo-layout-supporting-headers/)

The code for this post is in [https://github.com/behai-nguyen/polyglot_pdf/blob/main/pdf_05_header](https://github.com/behai-nguyen/polyglot_pdf/blob/main/pdf_05_header).

<strong>Headers</strong> are text rendered in larger font sizes, optionally in <strong>bold</strong>, <em>italic</em>, or <strong><em>bold italic</em></strong>. Following <code>Markdown</code>, we support  <a href="https://www.markdownguide.org/basic-syntax/#headings" title="Markdown Guide" target="_blank">six heading levels</a>: <code>#</code>..<code>######</code>. This article continues and extends the work from the <a href="https://behainguyen.wordpress.com/2025/12/27/rust-pdfs-exploring-layout-with-pango-and-cairo/" title="Rust: PDFs — Exploring Layout with Pango and Cairo" target="_blank">sixth article</a>. The final PDF produced here renders all natural headers using distinct, externally configured font settings.

9. [Rust: PDFs — Pango and Cairo Layout — Supporting Bold, Italic, and Bold Italic Text](https://behainguyen.wordpress.com/2026/02/23/rust-pdfs-pango-and-cairo-layout-supporting-bold-italic-and-bold-italic-text/)

The code for this post is in [https://github.com/behai-nguyen/polyglot_pdf/blob/main/pdf_06_text_styling](https://github.com/behai-nguyen/polyglot_pdf/blob/main/pdf_06_text_styling).

Implementing support for <strong>bold</strong>, <em>italic</em>, and <strong><em>bold italic</em></strong> text in paragraphs. Following <code>Markdown</code>, <a href="https://www.markdownguide.org/basic-syntax/#emphasis" title="Markdown Guide" target="_blank">these three indicators</a> — <code>**</code>, <code>*</code>, and <code>***</code> — are used. Adjacent and nested <code>Markdown</code> syntaxes, as well as escapes such as <code>\*</code> and <code>\\</code>, are supported. This article continues and extends the work from the <a href="https://behainguyen.wordpress.com/2026/01/30/rust-pdfs-pango-and-cairo-layout-supporting-headers/" title="Rust: PDFs — Exploring Layout with Pango and Cairo" target="_blank">eighth article</a>. In addition to rendering all natural headers, the final PDF now styles paragraph text according to the <code>Markdown</code> instructions in the source text file.