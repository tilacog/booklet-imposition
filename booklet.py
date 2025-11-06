#!/usr/bin/env python3
"""
booklet.py – prepare a PDF for booklet printing

This script takes a PDF file and transforms it into a booklet-ready format by:
1. Adding bleed to pages using pdfcrop (optional)
2. Padding the PDF to ensure page count is appropriate for binding
3. Splitting into signatures/sections (optional, for multi-signature binding)
4. Imposing the pages in booklet order using pdfbook2
5. Merging signatures back together (if applicable)

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


def pad_pdf(input_path: str, output_path: str, signature_size: int = 0) -> int:
    """Pad a PDF file to ensure total pages is a multiple of 4 (or signature_size if specified).

    Booklet printing requires a page count divisible by 4 (each sheet has 4 pages
    when folded: front-left, front-right, back-left, back-right). When using signatures,
    the page count must be divisible by (signature_size * number_of_signatures).

    Args:
        input_path: Path to the input PDF file
        output_path: Path where the padded PDF will be saved
        signature_size: Pages per signature (0 = treat as single booklet, pad to multiple of 4)

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
    # For signatures: pad to multiple of signature_size
    # For single booklet: pad to multiple of 4
    if signature_size > 0:
        # When using signatures, each signature must have exactly signature_size pages
        # and signature_size must be divisible by 4
        to_add = (signature_size - (total % signature_size)) % signature_size
    else:
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


# ---------- Step 3: Split into signatures ----------


def split_into_signatures(
    input_path: str, output_dir: str, signature_size: int, total_pages: int
) -> list[str]:
    """Split a PDF into multiple signature PDFs.

    Each signature will contain exactly signature_size pages.

    Args:
        input_path: Path to the input PDF file
        output_dir: Directory where signature PDFs will be saved
        signature_size: Number of pages per signature
        total_pages: Total number of pages in the input PDF

    Returns:
        List of paths to the created signature PDFs

    Raises:
        SystemExit: If splitting fails
    """
    num_signatures = total_pages // signature_size

    if total_pages % signature_size != 0:
        print(
            f"ERROR: Total pages ({total_pages}) is not divisible by signature size ({signature_size})",
            file=sys.stderr,
        )
        sys.exit(1)

    print(f"Splitting into {num_signatures} signature(s) of {signature_size} pages each...")

    signature_paths = []

    for sig_num in range(num_signatures):
        start_page = sig_num * signature_size + 1
        end_page = start_page + signature_size - 1

        output_path = os.path.join(output_dir, f"signature_{sig_num + 1:03d}.pdf")

        # Use ghostscript to extract page range
        gs_cmd = [
            "gs",
            "-q",
            "-dNOPAUSE",
            "-dBATCH",
            "-sDEVICE=pdfwrite",
            f"-dFirstPage={start_page}",
            f"-dLastPage={end_page}",
            f"-sOutputFile={output_path}",
            input_path,
        ]

        try:
            subprocess.run(gs_cmd, stderr=subprocess.DEVNULL, check=True)
        except subprocess.CalledProcessError:
            print(
                f"ERROR: Failed to extract signature {sig_num + 1}", file=sys.stderr
            )
            sys.exit(1)

        if not os.path.isfile(output_path):
            print(
                f"ERROR: Failed to create signature {sig_num + 1}", file=sys.stderr
            )
            sys.exit(1)

        signature_paths.append(output_path)
        print(
            f"  Created signature {sig_num + 1}/{num_signatures}: pages {start_page}-{end_page}"
        )

    return signature_paths


# ---------- Step 4: Booklet impose ----------


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


# ---------- Step 5: Merge signatures ----------


def merge_signatures(signature_paths: list[str], output_path: str) -> None:
    """Merge multiple imposed signature PDFs into a single PDF.

    Args:
        signature_paths: List of paths to imposed signature PDFs
        output_path: Path where the merged PDF will be saved

    Raises:
        SystemExit: If merging fails
    """
    print(f"Merging {len(signature_paths)} signature(s) into final booklet...")

    # Use ghostscript to merge PDFs
    gs_cmd = [
        "gs",
        "-q",
        "-dNOPAUSE",
        "-dBATCH",
        "-sDEVICE=pdfwrite",
        f"-sOutputFile={output_path}",
    ] + signature_paths

    try:
        subprocess.run(gs_cmd, stderr=subprocess.DEVNULL, check=True)
    except subprocess.CalledProcessError:
        print("ERROR: Failed to merge signatures", file=sys.stderr)
        sys.exit(1)

    if not os.path.isfile(output_path):
        print("ERROR: Failed to create merged booklet", file=sys.stderr)
        sys.exit(1)

    print(f"Signatures merged: {output_path}")


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
  %(prog)s input.pdf --signature 16              # 16-page signatures
  %(prog)s input.pdf -s 32 -p a4paper            # 32-page signatures on A4

Paper sizes:
  Common values include: a4paper (default), letterpaper, a5paper, legalpaper

Bleed:
  Bleed amount in points (72 points = 1 inch, ~3 points = 1mm)
  Use positive values to add bleed around content (e.g., 9 for ~3mm bleed)

Signatures:
  A signature (or section) is a group of folded pages in bookbinding.
  Instead of one large booklet, the PDF is split into multiple smaller
  signatures that are then gathered and bound together.
  Signature size must be divisible by 4. Common values: 8, 16, 32
  Use 0 (default) to treat the entire PDF as a single booklet.
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

    parser.add_argument(
        "-s",
        "--signature",
        dest="signature_size",
        type=int,
        default=0,
        metavar="PAGES",
        help="Pages per signature/section (default: 0, treat entire PDF as one booklet). Common values: 8, 16, 32",
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

    # Validate signature size
    if args.signature_size < 0:
        print("ERROR: Signature size must be a non-negative integer", file=sys.stderr)
        sys.exit(1)

    if args.signature_size > 0 and args.signature_size % 4 != 0:
        print(
            f"ERROR: Signature size ({args.signature_size}) must be divisible by 4",
            file=sys.stderr,
        )
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

    # Phase 2: Pad the PDF to appropriate page count
    print("=== Phase 2: Padding PDF ===")
    total = pad_pdf(str(bleed_pdf), str(padded_pdf), args.signature_size)
    print(f"Total pages after padding: {total}")
    print()

    # Choose workflow based on whether signatures are requested
    if args.signature_size > 0:
        # Signature-based workflow
        print(f"=== Phase 3: Splitting into signatures ({args.signature_size} pages each) ===")

        # Create temporary directory for signature files
        sig_dir = pdf_dir / f"{pdf_stem}_signatures"
        sig_dir.mkdir(exist_ok=True)

        # Split into signatures
        signature_paths = split_into_signatures(
            str(padded_pdf), str(sig_dir), args.signature_size, total
        )
        print()

        # Phase 4: Impose each signature
        print("=== Phase 4: Imposing signatures ===")
        imposed_paths = []

        for i, sig_path in enumerate(signature_paths, 1):
            sig_filename = Path(sig_path).name
            imposed_path = str(sig_dir / f"imposed_{sig_filename}")

            print(f"Imposing signature {i}/{len(signature_paths)}...")
            impose_booklet(sig_path, imposed_path, args.paper_size)

            imposed_paths.append(imposed_path)

        print()

        # Phase 5: Merge imposed signatures
        print("=== Phase 5: Merging signatures ===")
        merge_signatures(imposed_paths, str(booklet_pdf))
        print()

        # Clean up signature files
        print("Cleaning up signature files...")
        shutil.rmtree(sig_dir)

    else:
        # Single booklet workflow (original behavior)
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
    if args.signature_size > 0:
        num_signatures = total // args.signature_size
        print(f"  Signatures: {num_signatures} × {args.signature_size} pages")


if __name__ == "__main__":
    main()
