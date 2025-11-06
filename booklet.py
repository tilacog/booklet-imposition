#!/usr/bin/env python3
"""
booklet.py – prepare a PDF for booklet printing

This script takes a PDF file and transforms it into a booklet-ready format by:
1. Adding bleed to pages using pdfcrop (optional)
2. Padding the PDF to ensure page count is a multiple of 4
3. Imposing the pages in booklet order using pdfbook2

Dependencies:
  - ghostscript (gs)
  - pdfcrop (from texlive-extra-utils, only needed if bleed > 0)
  - pdfbook2 (from texlive-extra-utils)
"""

import argparse
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

# ---------- Utilities ----------


def check_dependency(cmd: str) -> None:
    """Check if a command exists in PATH.

    Args:
        cmd: Command name to check

    Raises:
        SystemExit: If command is not found
    """
    if not shutil.which(cmd):
        print(f"ERROR: Missing dependency: {cmd}", file=sys.stderr)
        sys.exit(1)


# ---------- Step 1: Add Bleed ----------


def add_bleed(input_path: str, output_path: str, bleed_amount: int) -> None:
    """Add bleed to a PDF using pdfcrop with positive margins.

    Bleed is extra content area that extends beyond the final trim size,
    ensuring no white edges appear after cutting.

    Args:
        input_path: Path to the input PDF file
        output_path: Path where the PDF with bleed will be saved
        bleed_amount: Bleed in points (72 points = 1 inch, ~3pt = 1mm)

    Raises:
        SystemExit: If bleed addition fails
    """
    if bleed_amount == 0:
        print("Bleed disabled (0pt) - copying file...")
        shutil.copy2(input_path, output_path)
        return

    check_dependency("pdfcrop")

    print(f"Adding {bleed_amount}pt bleed to all sides...")

    # Positive margins add bleed by expanding the page
    margins = f"{bleed_amount} {bleed_amount} {bleed_amount} {bleed_amount}"

    try:
        subprocess.run(
            ["pdfcrop", "--margins", margins, input_path, output_path],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=True,
        )
    except subprocess.CalledProcessError:
        print("ERROR: Failed to add bleed to PDF", file=sys.stderr)
        sys.exit(1)

    if not os.path.isfile(output_path):
        print("ERROR: Failed to add bleed to PDF", file=sys.stderr)
        sys.exit(1)

    print(f"Bleed added: {output_path}")


# ---------- Step 2: Pad ----------


def pad_pdf(input_path: str, output_path: str) -> int:
    """Pad a PDF file to ensure total pages is a multiple of 4.

    Booklet printing requires a page count divisible by 4 (each sheet has 4 pages
    when folded: front-left, front-right, back-left, back-right). This function
    adds blank pages as needed.

    Args:
        input_path: Path to the input PDF file
        output_path: Path where the padded PDF will be saved

    Returns:
        Total number of pages after padding

    Raises:
        SystemExit: If padding fails or page count cannot be determined
    """
    print("Counting pages in input PDF...")

    # Count pages using ghostscript
    # Note: -dNOSAFER is required to read files, but we only read user-provided input
    gs_cmd = [
        "gs",
        "-q",
        "-dNOSAFER",
        "-dNODISPLAY",
        "-c",
        f"({input_path}) (r) file runpdfbegin pdfpagecount = quit",
    ]

    try:
        result = subprocess.run(gs_cmd, capture_output=True, text=True, check=True)
        total_str = result.stdout.strip()
    except subprocess.CalledProcessError:
        print("ERROR: Could not determine page count from PDF", file=sys.stderr)
        sys.exit(1)

    # Validate the page count is a number
    if not re.match(r"^\d+$", total_str):
        print("ERROR: Could not determine page count from PDF", file=sys.stderr)
        sys.exit(1)

    total = int(total_str)

    # Calculate how many blank pages are needed
    # Formula: (4 - (total % 4)) % 4 returns 0-3 blank pages needed
    to_add = (4 - (total % 4)) % 4

    print(f"Original pages: {total}")

    if to_add == 0:
        print("Multiple of 4 already - no padding required.")
        shutil.copy2(input_path, output_path)
    else:
        print(f"Adding {to_add} blank page(s)...")

        # Use ghostscript to append blank pages
        # We read the original PDF and then add blank pages using PostScript commands
        blank_pages = "<</PageSize[595 842]>> setpagedevice showpage " * to_add

        gs_pad_cmd = [
            "gs",
            "-q",
            "-dNOPAUSE",
            "-dBATCH",
            "-sDEVICE=pdfwrite",
            f"-sOutputFile={output_path}",
            input_path,
            "-c",
            blank_pages,
        ]

        try:
            subprocess.run(gs_pad_cmd, stderr=subprocess.DEVNULL, check=True)
        except subprocess.CalledProcessError:
            print("ERROR: Failed to create padded PDF", file=sys.stderr)
            sys.exit(1)

        if not os.path.isfile(output_path):
            print("ERROR: Failed to create padded PDF", file=sys.stderr)
            sys.exit(1)

    new_total = total + to_add
    print(f"Padded PDF: {output_path} ({new_total} pages)")

    return new_total


# ---------- Step 3: Booklet impose ----------


def impose_booklet(input_path: str, output_path: str, paper_size: str) -> None:
    """Transform a padded PDF into booklet format using pdfbook2.

    This function rearranges pages for duplex (double-sided) booklet printing.
    Pages are reordered so that when printed on both sides and folded, they
    appear in the correct sequence.

    Args:
        input_path: Path to the padded PDF file (must have page count divisible by 4)
        output_path: Path where the booklet PDF will be saved
        paper_size: Paper size specification (e.g., 'a4paper', 'letterpaper')

    Raises:
        SystemExit: If booklet creation fails
    """
    check_dependency("pdfbook2")

    print("Creating booklet...")

    # Detect modern or legacy pdfbook2 syntax
    # Modern versions support --outfile flag, legacy versions don't
    try:
        help_result = subprocess.run(
            ["pdfbook2", "--help"], capture_output=True, text=True
        )
        help_output = help_result.stdout + help_result.stderr
    except subprocess.CalledProcessError:
        help_output = ""

    if "--outfile" in help_output:
        # Modern pdfbook2: explicitly specify output file
        try:
            subprocess.run(
                [
                    "pdfbook2",
                    "--paper",
                    paper_size,
                    "--no-crop",
                    input_path,
                    "--outfile",
                    output_path,
                ],
                check=True,
            )
        except subprocess.CalledProcessError:
            print("ERROR: pdfbook2 failed", file=sys.stderr)
            sys.exit(1)
    else:
        # Legacy pdfbook2: auto-generates output filename
        try:
            subprocess.run(
                ["pdfbook2", "--paper", paper_size, "--no-crop", input_path], check=True
            )
        except subprocess.CalledProcessError:
            print("ERROR: pdfbook2 failed", file=sys.stderr)
            sys.exit(1)

        # Handle legacy pdfbook2 which writes <stem>-book.pdf
        input_dir = os.path.dirname(input_path)
        input_stem = Path(input_path).stem
        candidate = os.path.join(input_dir, f"{input_stem}-book.pdf")

        if os.path.isfile(candidate):
            shutil.move(candidate, output_path)

    # Verify the booklet was created successfully
    if os.path.isfile(output_path):
        print(f"Booklet created: {output_path}")
    else:
        print("ERROR: Could not find output booklet file.", file=sys.stderr)
        sys.exit(1)


# ---------- Main ----------


def main() -> None:
    """Main entry point for the booklet preparation tool."""
    parser = argparse.ArgumentParser(
        description="Prepare a PDF for booklet printing",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s input.pdf
  %(prog)s input.pdf --paper letterpaper
  %(prog)s input.pdf --paper a5paper --bleed 9
  %(prog)s input.pdf -p letterpaper -b 9

Paper sizes:
  Common values include: a4paper (default), letterpaper, a5paper, legalpaper

Bleed:
  Bleed amount in points (72 points = 1 inch, ~3 points = 1mm)
  Use positive values to add bleed around content (e.g., 9 for ~3mm bleed)
        """,
    )

    parser.add_argument("input", metavar="INPUT_PDF", help="Path to input PDF file")

    parser.add_argument(
        "-p",
        "--paper",
        dest="paper_size",
        default="a4paper",
        metavar="SIZE",
        help="Paper size for booklet (default: a4paper)",
    )

    parser.add_argument(
        "-b",
        "--bleed",
        dest="bleed",
        type=int,
        default=0,
        metavar="POINTS",
        help="Bleed amount in points (default: 0, disabled)",
    )

    parser.add_argument("-v", "--version", action="version", version="%(prog)s 2.0")

    args = parser.parse_args()

    # Validate input file exists
    if not os.path.isfile(args.input):
        print(f"ERROR: File not found: {args.input}", file=sys.stderr)
        sys.exit(1)

    # Validate bleed is non-negative
    if args.bleed < 0:
        print("ERROR: Bleed must be a non-negative integer", file=sys.stderr)
        sys.exit(1)

    # Check for required system dependencies
    check_dependency("gs")
    check_dependency("pdfbook2")
    if args.bleed > 0:
        check_dependency("pdfcrop")

    # Generate output filenames based on input
    pdf_path = Path(args.input)
    pdf_dir = pdf_path.parent
    pdf_stem = pdf_path.stem

    bleed_pdf = pdf_dir / f"{pdf_stem}_bleed.pdf"
    padded_pdf = pdf_dir / f"{pdf_stem}_padded.pdf"
    booklet_pdf = pdf_dir / f"{pdf_stem}_booklet.pdf"

    # Phase 1: Add bleed to the PDF
    print("=== Phase 1: Adding bleed ===")
    add_bleed(str(args.input), str(bleed_pdf), args.bleed)
    print()

    # Phase 2: Pad the PDF to a multiple of 4 pages
    print("=== Phase 2: Padding PDF ===")
    total = pad_pdf(str(bleed_pdf), str(padded_pdf))
    print(f"Total pages after padding: {total}")
    print()

    # Phase 3: Impose pages into booklet format
    print("=== Phase 3: Creating booklet ===")
    impose_booklet(str(padded_pdf), str(booklet_pdf), args.paper_size)
    print()

    # Clean up intermediate files
    if bleed_pdf.exists():
        bleed_pdf.unlink()
        print(f"Cleaned up intermediate file: {bleed_pdf}")

    if padded_pdf.exists():
        padded_pdf.unlink()
        print(f"Cleaned up intermediate file: {padded_pdf}")

    # Display final results
    print()
    print("Done.")
    print(f"  Booklet: {booklet_pdf}")


if __name__ == "__main__":
    main()
