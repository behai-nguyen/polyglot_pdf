<#
25/02/2026.

To run from CLI: powershell.exe -File F:\rust\polyglot_pdf\set_pkg_config_path.ps1

When working with Pango, HarfBuzz, Cairo, etc., Visual Studio Code’s rust-analyzer
plugin—as well as the compiler and linker—requires the logged-in user’s
PKG_CONFIG_PATH environment variable to be set correctly.

After running this script, the PKG_CONFIG_PATH environment variable will appear under
“Environment Variables” → “User variables for behai”, where “behai” is the logged-in
user. It is recommended to manually verify that it has been set as intended.

After setting or resetting PKG_CONFIG_PATH, restart Visual Studio Code.

The paths listed here reflect my headless installation; you will need to update them
to match your own setup.

The GDK4 library also uses Pango, HarfBuzz, Cairo, etc., but it ships with its own
build. PKG_CONFIG_PATH **must** point to GDK4’s pkgconfig directory when working with
GDK4.

When both setups exist on the same system, it is essential that PKG_CONFIG_PATH
**always be set** to the correct library stack for the project you are working on.
Otherwise, the project may compile successfully but fail immediately at runtime.	
#>

[System.Environment]::SetEnvironmentVariable("PKG_CONFIG_PATH", "C:\PF\pango\dist\lib\pkgconfig;C:\PF\cairo-1.18.4\dist\lib\pkgconfig;C:\PF\vcpkg\installed\x64-windows\lib\pkgconfig;C:\PF\harfbuzz\dist\lib\pkgconfig;C:\PF\fribidi\dist\lib\pkgconfig", "User")
