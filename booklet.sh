#!/usr/bin/env bash
#
# booklet.sh – prepare a PDF for booklet printing (single-sided workflow)
#
# This script takes a PDF file and transforms it into a booklet-ready format by:
# 1. Adding bleed to pages using pdfcrop (optional)
# 2. Padding the PDF to ensure page count is a multiple of 4
# 3. Imposing the pages in booklet order using pdfbook2
#
# Dependencies:
#   - ghostscript (gs)
#   - pdfcrop (from texlive-extra-utils, only needed if bleed > 0)
#   - pdfbook2 (from texlive-extra-utils)
#
# Usage:
#   ./booklet.sh input.pdf [paper-size] [bleed]
#
# Arguments:
#   input.pdf   - Path to input PDF file
#   paper-size  - Optional paper size (default: a4paper)
#                 Common values: a4paper, letterpaper, a5paper
#   bleed       - Optional bleed amount in points (default: 0)
#                 Use positive values to add bleed (e.g., 9 for 3mm)
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

# ---------- Step 1: Add Bleed ----------

# Add bleed to a PDF using pdfcrop with negative margins
#
# Bleed is extra content area that extends beyond the final trim size,
# ensuring no white edges appear after cutting.
#
# Args:
#   $1 - input_path: Path to the input PDF file
#   $2 - output_path: Path where the PDF with bleed will be saved
#   $3 - bleed_amount: Bleed in points (72 points = 1 inch, ~3pt = 1mm)
#
add_bleed() {
    local input_path="$1"
    local output_path="$2"
    local bleed_amount="$3"

    if [[ "$bleed_amount" -eq 0 ]]; then
        echo "Bleed disabled (0pt) - copying file..."
        cp "$input_path" "$output_path"
        return
    fi

    check_dependency "pdfcrop"

    echo "Adding ${bleed_amount}pt bleed to all sides..."

    # Positive margins add bleed by expanding the page
    # pdfcrop with positive values adds space around content
    pdfcrop --margins "$bleed_amount $bleed_amount $bleed_amount $bleed_amount" \
            "$input_path" "$output_path" > /dev/null 2>&1

    if [[ ! -f "$output_path" ]]; then
        echo "ERROR: Failed to add bleed to PDF" >&2
        exit 1
    fi

    echo "Bleed added: $output_path"
}

# ---------- Step 2: Pad ----------

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

# ---------- Step 3: Booklet impose ----------

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
        pdfbook2 --paper "$paper_size" --no-crop \
                 "$input_path" --outfile "$output_path"
    else
        # Legacy pdfbook2: auto-generates output filename
        pdfbook2 --paper "$paper_size" --no-crop "$input_path"

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
        echo "Usage: $0 input.pdf [paper-size] [bleed]" >&2
        echo "" >&2
        echo "Arguments:" >&2
        echo "  input.pdf   - Path to input PDF file" >&2
        echo "  paper-size  - Optional paper size (default: a4paper)" >&2
        echo "                Common values: a4paper, letterpaper, a5paper" >&2
        echo "  bleed       - Optional bleed in points (default: 0)" >&2
        echo "                Example: 9 for ~3mm bleed" >&2
        exit 1
    fi

    local pdf_in="$1"
    local paper="${2:-a4paper}"
    local bleed="${3:-0}"

    # Validate input file exists
    if [[ ! -f "$pdf_in" ]]; then
        echo "ERROR: File not found: $pdf_in" >&2
        exit 1
    fi

    # Validate bleed is a number
    if [[ ! "$bleed" =~ ^[0-9]+$ ]]; then
        echo "ERROR: Bleed must be a non-negative integer" >&2
        exit 1
    fi

    # Check for required system dependencies
    check_dependency "gs"
    check_dependency "pdfbook2"
    if [[ "$bleed" -gt 0 ]]; then
        check_dependency "pdfcrop"
    fi

    # Generate output filenames based on input
    local pdf_dir
    local pdf_stem
    pdf_dir=$(dirname "$pdf_in")
    pdf_stem=$(basename "$pdf_in" .pdf)

    local bleed_pdf="${pdf_dir}/${pdf_stem}_bleed.pdf"
    local padded="${pdf_dir}/${pdf_stem}_padded.pdf"
    local booklet="${pdf_dir}/${pdf_stem}_booklet.pdf"

    # Phase 1: Add bleed to the PDF
    echo "=== Phase 1: Adding bleed ==="
    add_bleed "$pdf_in" "$bleed_pdf" "$bleed"
    echo ""

    # Phase 2: Pad the PDF to a multiple of 4 pages
    echo "=== Phase 2: Padding PDF ==="
    local total
    total=$(pad_pdf "$bleed_pdf" "$padded")
    echo "Total pages after padding: $total"
    echo ""

    # Phase 3: Impose pages into booklet format
    echo "=== Phase 3: Creating booklet ==="
    impose_booklet "$padded" "$booklet" "$paper"
    echo ""

    # Clean up intermediate files
    if [[ -f "$bleed_pdf" ]]; then
        rm "$bleed_pdf"
        echo "Cleaned up intermediate file: $bleed_pdf"
    fi
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
