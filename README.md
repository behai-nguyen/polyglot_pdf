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