# PDF Booklet Maker

A Python tool that transforms PDFs into printable booklet format with optional bleed and signature support. The script:
1. Adds bleed using `pdfcrop` (optional)
2. Pads to appropriate page count
3. Splits into signatures/sections (optional, for multi-signature binding)
4. Imposes pages for booklet printing using `pdfbook2`
5. Merges signatures back together (if applicable)

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

./booklet.py input.pdf                           # A4, no bleed, single booklet (default)
./booklet.py input.pdf -p letterpaper            # Custom paper size
./booklet.py input.pdf -p a4paper -b 9           # A4 with 9pt (~3mm) bleed
./booklet.py input.pdf --paper letterpaper --bleed 18  # Letter with 18pt (~6mm) bleed
./booklet.py input.pdf --signature 16            # Split into 16-page signatures
./booklet.py input.pdf -s 32 -p a4paper -b 9     # 32-page signatures, A4, with bleed

# Get help
./booklet.py --help
```

### Options

- `INPUT_PDF` - Path to input PDF file (required)
- `-p, --paper SIZE` - Paper size (default: a4paper). Common values: a4paper, letterpaper, a5paper, legalpaper
- `-b, --bleed POINTS` - Bleed in points (default: 0). Example: 9 for ~3mm bleed
- `-s, --signature PAGES` - Pages per signature/section (default: 0 = single booklet). Common values: 8, 16, 32. Must be divisible by 4
- `-h, --help` - Show help message and exit
- `-v, --version` - Show version and exit

## Features

**Bleed Support:** Adds extra content area beyond the trim size to prevent white edges after cutting. Useful for professional printing. Bleed is disabled by default (0pt).

**Signature Support:** Split PDFs into multiple signatures (sections) for traditional bookbinding. Instead of one large booklet, the PDF is divided into smaller signatures that are gathered and bound together. This is essential for larger books and provides flexibility in binding methods.

**Pure Python:** Uses only Python standard library with no third-party dependencies. Calls external tools (ghostscript, pdfbook2, pdfcrop) via subprocess.

**Proper CLI:** Built with argparse for a professional command-line experience with helpful error messages and comprehensive help documentation.

## How It Works

### Single Booklet Mode (default)

The script processes PDFs in three phases:

1. **Add Bleed**: Uses `pdfcrop` to expand page margins, adding bleed space around content
2. **Pad Pages**: Uses `ghostscript` to add blank pages if needed to ensure page count is divisible by 4
3. **Impose Booklet**: Uses `pdfbook2` to rearrange pages for duplex booklet printing

### Signature Mode (with `--signature`)

When signatures are enabled, the workflow expands to five phases:

1. **Add Bleed**: Uses `pdfcrop` to expand page margins (optional)
2. **Pad Pages**: Ensures page count is divisible by signature size
3. **Split into Signatures**: Divides the PDF into multiple signature PDFs using `ghostscript`
4. **Impose Each Signature**: Each signature is imposed separately using `pdfbook2`
5. **Merge Signatures**: All imposed signatures are combined into the final booklet using `ghostscript`

Output: `<input>_booklet.pdf`

## References

- [Section (bookbinding) - Wikipedia](https://en.wikipedia.org/wiki/Section_(bookbinding)) - Information about signatures/sections in bookbinding

## License

MIT
