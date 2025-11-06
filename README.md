# PDF Booklet Maker

A Python tool that transforms PDFs into printable booklet format with optional bleed support. The script:
1. Adds bleed using `pdfcrop` (optional)
2. Pads to multiples of 4 pages
3. Imposes pages for booklet printing using `pdfbook2`

## Requirements

- Python 3.6+
- ghostscript
- pdfbook2 (from texlive-extra-utils)
- pdfcrop (from texlive-extra-utils, only needed if using bleed > 0)

**Arch Linux:**
```bash
sudo pacman -S python texlive-core texlive-latexextra texlive-fontsextra texlive-bin texlive-extra-utils ghostscript
```

**Ubuntu/Debian:**
```bash
sudo apt install python3 texlive-extra-utils texlive-latex-extra ghostscript
```

## Usage

```bash
chmod +x booklet.py

./booklet.py input.pdf                           # A4, no bleed (default)
./booklet.py input.pdf -p letterpaper            # Custom paper size
./booklet.py input.pdf -p a4paper -b 9           # A4 with 9pt (~3mm) bleed
./booklet.py input.pdf --paper letterpaper --bleed 18  # Letter with 18pt (~6mm) bleed

# Get help
./booklet.py --help
```

### Options

- `INPUT_PDF` - Path to input PDF file (required)
- `-p, --paper SIZE` - Paper size (default: a4paper). Common values: a4paper, letterpaper, a5paper, legalpaper
- `-b, --bleed POINTS` - Bleed in points (default: 0). Example: 9 for ~3mm bleed
- `-h, --help` - Show help message and exit
- `-v, --version` - Show version and exit

## Features

**Bleed Support:** Adds extra content area beyond the trim size to prevent white edges after cutting. Useful for professional printing. Bleed is disabled by default (0pt).

**Pure Python:** Uses only Python standard library with no third-party dependencies. Calls external tools (ghostscript, pdfbook2, pdfcrop) via subprocess.

**Proper CLI:** Built with argparse for a professional command-line experience with helpful error messages and comprehensive help documentation.

## How It Works

The script processes PDFs in three phases:

1. **Add Bleed**: Uses `pdfcrop` to expand page margins, adding bleed space around content
2. **Pad Pages**: Uses `ghostscript` to add blank pages if needed to ensure page count is divisible by 4
3. **Impose Booklet**: Uses `pdfbook2` to rearrange pages for duplex booklet printing

Output: `<input>_booklet.pdf`

## License

MIT
