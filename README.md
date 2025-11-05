# PDF Booklet Maker

Pads PDFs to multiples of 4 pages and imposes them into printable booklet format using `pdfbook2`. 

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
./booklet.sh input.pdf              # A4 (default)
./booklet.sh input.pdf letterpaper  # Custom paper size
```

## License

MIT
