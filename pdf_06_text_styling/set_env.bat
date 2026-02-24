@echo off
rem 09/12/2025

rem C:\PF\harfbuzz\dist\bin\[harfbuzz.dll, harfbuzz-subset.dll, harfbuzz-gobject.dll]
set PATH=C:\PF\harfbuzz\dist\bin\;%PATH%

rem C:\PF\vcpkg\installed\x64-windows\bin\freetype.dll
set PATH=C:\PF\vcpkg\installed\x64-windows\bin\;%PATH%

set PATH=C:\PF\pango\dist\bin;C:\PF\cairo-1.18.4\dist\bin;C:\PF\fribidi\dist\bin;%PATH%

rem Discussed in https://behainguyen.wordpress.com/2025/12/19/rust-pdfs-build-and-install-pango-and-associated-libraries/#windows-build-install-pango
rem Should also set in user environment variable so that rust-analyzer can work correctly.
set PKG_CONFIG_PATH=C:\PF\pango\dist\lib\pkgconfig;C:\PF\cairo-1.18.4\dist\lib\pkgconfig;C:\PF\vcpkg\installed\x64-windows\lib\pkgconfig;C:\PF\harfbuzz\dist\lib\pkgconfig;C:\PF\fribidi\dist\lib\pkgconfig
