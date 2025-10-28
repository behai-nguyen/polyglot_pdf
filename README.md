<!-- 29/10/2025 -->
# Polyglot PDF

Explorations in PDF generation using Rust, with support for multilingual text, font subsetting, and HarfBuzz integration.

## Related post(s)

1. [Rust FFI “Adventure” with the HarfBuzz Text Shaping Engine](https://behainguyen.wordpress.com/2025/10/28/rust-ffi-adventure-with-the-harfbuzz-text-shaping-engine/)

The code for the above post is in [https://github.com/behai-nguyen/polyglot_pdf/tree/main/harfbuzz_ffi](https://github.com/behai-nguyen/polyglot_pdf/tree/main/harfbuzz_ffi).

Rust FFI, or <a href="https://doc.rust-lang.org/nomicon/ffi.html" title="Foreign Function Interface" target="_blank">Foreign Function Interface</a>, is a mechanism that allows Rust code to interact with programs written in other languages, such as C and C-compatible languages. The <a href="https://en.wikipedia.org/wiki/HarfBuzz" title="HarfBuzz" target="_blank">HarfBuzz</a> text shaping engine is written in C++.

In this article, we describe how to build the <code>HarfBuzz</code> text shaping engine on both Windows and Ubuntu. We then demonstrate how to write simple Rust code that calls the <code>hb_version_string()</code> function from <code>HarfBuzz</code> using FFI.
