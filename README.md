# PDF Booklet Maker

Transforms PDFs into printable booklet format with optional bleed support. The script:
1. Adds bleed using `pdfcrop` (optional)
2. Pads to multiples of 4 pages
3. Imposes pages for booklet printing using `pdfbook2`

## Requirements

**Arch Linux:**
```bash
sudo pacman -S texlive-core texlive-latexextra texlive-fontsextra texlive-bin texlive-extra-utils ghostscript
```

**Ubuntu/Debian:**
```bash
sudo apt install texlive-extra-utils texlive-latex-extra ghostscript
```

## Usage

```bash
chmod +x booklet.sh
./booklet.sh input.pdf                    # A4, no bleed (default)
./booklet.sh input.pdf letterpaper        # Custom paper size, no bleed
./booklet.sh input.pdf a4paper 9          # A4 with 9pt (~3mm) bleed
./booklet.sh input.pdf letterpaper 18     # Letter with 18pt (~6mm) bleed
```

**Arguments:**
- `input.pdf` - Path to input PDF file
- `paper-size` - Optional paper size (default: a4paper). Common values: a4paper, letterpaper, a5paper
- `bleed` - Optional bleed in points (default: 0). Use positive values like 9 for ~3mm bleed

## Features

**Bleed Support:** Adds extra content area beyond the trim size to prevent white edges after cutting. Useful for professional printing. Bleed is disabled by default (0pt) to maintain backward compatibility.

## License

MIT
