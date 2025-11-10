#!/usr/bin/env python3
"""
Calculate PDF page numbers to print for signature printing in traditional bookbinding.

The input PDF is already imposed (2 content pages per sheet side).
Each signature contains 16 content pages = 8 PDF pages (4 sheets × 2 sides).

For printing even-numbered PDF pages in reverse, you print:
- First pass: odd PDF pages (1, 3, 5, 7)
- Flip the stack
- Second pass: even PDF pages in reverse (8, 6, 4, 2)
"""

import sys


def calculate_signatures(total_pdf_pages):
    """
    Calculate which PDF pages to print for each signature.

    Args:
        total_pdf_pages: Total number of pages in the imposed PDF

    Returns:
        List of signatures with PDF page numbers to print
    """
    PDF_PAGES_PER_SIGNATURE = 8  # 4 sheets × 2 sides

    # Pad to nearest multiple of 8
    padded_pages = ((total_pdf_pages + PDF_PAGES_PER_SIGNATURE - 1) // PDF_PAGES_PER_SIGNATURE) * PDF_PAGES_PER_SIGNATURE

    signatures = []

    for sig_num in range(padded_pages // PDF_PAGES_PER_SIGNATURE):
        start_pdf_page = sig_num * PDF_PAGES_PER_SIGNATURE + 1
        end_pdf_page = start_pdf_page + PDF_PAGES_PER_SIGNATURE - 1

        # First pass: odd PDF pages in order
        odd_pages = list(range(start_pdf_page, end_pdf_page + 1, 2))

        # Second pass: even PDF pages in reverse
        even_pages = list(range(end_pdf_page if end_pdf_page % 2 == 0 else end_pdf_page - 1,
                                start_pdf_page - 1, -2))

        signatures.append({
            'number': sig_num + 1,
            'odd_pages': odd_pages,
            'even_pages': even_pages
        })

    return signatures, padded_pages


def print_signatures(total_pdf_pages):
    """Print the PDF page numbers for all signatures."""
    signatures, padded_pages = calculate_signatures(total_pdf_pages)

    print(f"Total PDF pages: {total_pdf_pages}")
    if padded_pages != total_pdf_pages:
        print(f"Padded to: {padded_pages} pages ({padded_pages - total_pdf_pages} blank PDF pages needed)")
    print(f"Number of signatures: {len(signatures)}")
    print()

    for sig in signatures:
        ordinal = {1: 'st', 2: 'nd', 3: 'rd'}.get(sig['number'], 'th')
        print(f"{sig['number']}{ordinal} signature:")
        print(f"  - {', '.join(map(str, sig['odd_pages']))}")
        print(f"  - {', '.join(map(str, sig['even_pages']))}")
        print()


def main():
    if len(sys.argv) != 2:
        print("Usage: signature_pages.py <total_pages>", file=sys.stderr)
        print("\nExample: signature_pages.py 64", file=sys.stderr)
        sys.exit(1)

    try:
        total_pages = int(sys.argv[1])
        if total_pages < 1:
            raise ValueError("Page count must be positive")
    except ValueError as e:
        print(f"Error: Invalid page count - {e}", file=sys.stderr)
        sys.exit(1)

    print_signatures(total_pages)


if __name__ == "__main__":
    main()
