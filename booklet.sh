#!/usr/bin/env bash
#
# booklet.sh – prepare a PDF for booklet printing (single-sided workflow)
#
# This script takes a PDF file and transforms it into a booklet-ready format by:
# 1. Padding the PDF to ensure page count is a multiple of 4
# 2. Imposing the pages in booklet order using pdfbook2
#
# Dependencies:
#   - ghostscript (gs)
#   - pdfbook2 (from texlive-extra-utils)
#
# Usage:
#   ./booklet.sh input.pdf [paper-size]
#
# Arguments:
#   input.pdf   - Path to input PDF file
#   paper-size  - Optional paper size (default: a4paper)
#                 Common values: a4paper, letterpaper, a5paper
#
# Output:
#   <input>_booklet.pdf - Final booklet-ready PDF file
#

set -euo pipefail

# ---------- Utilities ----------

# Check if a command exists in PATH
check_dependency() {
    local cmd="$1"
    if ! command -v "$cmd" &> /dev/null; then
        echo "ERROR: Missing dependency: $cmd" >&2
        exit 1
    fi
}

# ---------- Step 1: Pad ----------

# Pad a PDF file to ensure total pages is a multiple of 4
#
# Booklet printing requires a page count divisible by 4 (each sheet has 4 pages
# when folded: front-left, front-right, back-left, back-right). This function
# adds blank pages as needed.
#
# Args:
#   $1 - input_path: Path to the input PDF file
#   $2 - output_path: Path where the padded PDF will be saved
#
# Returns:
#   Prints total number of pages after padding
pad_pdf() {
    local input_path="$1"
    local output_path="$2"

    echo "Counting pages in input PDF..."

    # Count pages using ghostscript
    # Note: -dNOSAFER is required to read files, but we only read user-provided input
    local total
    total=$(gs -q -dNOSAFER -dNODISPLAY -c "(${input_path}) (r) file runpdfbegin pdfpagecount = quit" 2>/dev/null)

    if [[ ! "$total" =~ ^[0-9]+$ ]]; then
        echo "ERROR: Could not determine page count from PDF" >&2
        exit 1
    fi

    # Calculate how many blank pages are needed
    # Formula: (4 - (total % 4)) % 4 returns 0-3 blank pages needed
    local to_add=$(( (4 - (total % 4)) % 4 ))

    echo "Original pages: $total"

    if [[ $to_add -eq 0 ]]; then
        echo "Multiple of 4 already - no padding required."
        # Just copy the file
        cp "$input_path" "$output_path"
    else
        echo "Adding $to_add blank page(s)..."

        # Use ghostscript to append blank pages
        # We read the original PDF and then add blank pages using PostScript commands
        local blank_pages=""
        for ((i=0; i<to_add; i++)); do
            blank_pages+="<</PageSize[595 842]>> setpagedevice showpage "
        done

        gs -q -dNOPAUSE -dBATCH -sDEVICE=pdfwrite \
           -sOutputFile="$output_path" \
           "$input_path" \
           -c "$blank_pages" 2>/dev/null

        if [[ ! -f "$output_path" ]]; then
            echo "ERROR: Failed to create padded PDF" >&2
            exit 1
        fi
    fi

    local new_total=$((total + to_add))
    echo "Padded PDF: $output_path ($new_total pages)"
    echo "$new_total"
}

# ---------- Step 2: Booklet impose ----------

# Transform a padded PDF into booklet format using pdfbook2
#
# This function rearranges pages for duplex (double-sided) booklet printing.
# Pages are reordered so that when printed on both sides and folded, they
# appear in the correct sequence.
#
# Args:
#   $1 - input_path: Path to the padded PDF file (must have page count divisible by 4)
#   $2 - output_path: Path where the booklet PDF will be saved
#   $3 - paper_size: Paper size specification (e.g., 'a4paper', 'letterpaper')
impose_booklet() {
    local input_path="$1"
    local output_path="$2"
    local paper_size="$3"

    check_dependency "pdfbook2"

    echo "Creating booklet..."

    # Detect modern or legacy pdfbook2 syntax
    # Modern versions support --outfile flag, legacy versions don't
    local help_output
    help_output=$(pdfbook2 --help 2>&1 || true)

    if echo "$help_output" | grep -q -- "--outfile"; then
        # Modern pdfbook2: explicitly specify output file
        pdfbook2 --short-edge --paper "$paper_size" --no-crop \
                 "$input_path" --outfile "$output_path"
    else
        # Legacy pdfbook2: auto-generates output filename
        pdfbook2 --short-edge --paper "$paper_size" --no-crop "$input_path"

        # Handle legacy pdfbook2 which writes <stem>-book.pdf
        local input_dir
        local input_stem
        input_dir=$(dirname "$input_path")
        input_stem=$(basename "$input_path" .pdf)
        local candidate="${input_dir}/${input_stem}-book.pdf"

        if [[ -f "$candidate" ]]; then
            mv "$candidate" "$output_path"
        fi
    fi

    # Verify the booklet was created successfully
    if [[ -f "$output_path" ]]; then
        echo "Booklet created: $output_path"
    else
        echo "ERROR: Could not find output booklet file." >&2
        exit 1
    fi
}

# ---------- Main ----------

main() {
    # Parse command-line arguments
    if [[ $# -lt 1 ]]; then
        echo "Usage: $0 input.pdf [paper-size]" >&2
        echo "" >&2
        echo "Arguments:" >&2
        echo "  input.pdf   - Path to input PDF file" >&2
        echo "  paper-size  - Optional paper size (default: a4paper)" >&2
        echo "                Common values: a4paper, letterpaper, a5paper" >&2
        exit 1
    fi

    local pdf_in="$1"
    local paper="${2:-a4paper}"

    # Validate input file exists
    if [[ ! -f "$pdf_in" ]]; then
        echo "ERROR: File not found: $pdf_in" >&2
        exit 1
    fi

    # Check for required system dependencies
    check_dependency "gs"
    check_dependency "pdfbook2"

    # Generate output filenames based on input
    local pdf_dir
    local pdf_stem
    pdf_dir=$(dirname "$pdf_in")
    pdf_stem=$(basename "$pdf_in" .pdf)

    local padded="${pdf_dir}/${pdf_stem}_padded.pdf"
    local booklet="${pdf_dir}/${pdf_stem}_booklet.pdf"

    # Phase 1: Pad the PDF to a multiple of 4 pages
    echo "=== Phase 1: Padding PDF ==="
    local total
    total=$(pad_pdf "$pdf_in" "$padded")
    echo "Total pages after padding: $total"
    echo ""

    # Phase 2: Impose pages into booklet format
    echo "=== Phase 2: Creating booklet ==="
    impose_booklet "$padded" "$booklet" "$paper"
    echo ""

    # Clean up intermediate padded file
    if [[ -f "$padded" ]]; then
        rm "$padded"
        echo "Cleaned up intermediate file: $padded"
    fi

    # Display final results
    echo ""
    echo "Done."
    echo "  Booklet: $booklet"
}

# Run main function
main "$@"
