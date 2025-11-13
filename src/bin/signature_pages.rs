//! Calculate PDF page numbers to print for signature printing in traditional bookbinding.
//!
//! When you have an imposed PDF (already processed with 2 content pages per sheet side),
//! this tool tells you exactly which PDF pages to print for duplex printing with even
//! pages in reverse.

use booklet_imposition::print_signatures;
use clap::Parser;
use std::process;

#[derive(Parser, Debug)]
#[command(name = "signature-pages")]
#[command(version, about = "Calculate PDF page numbers for signature printing")]
#[command(long_about = r"Calculate which PDF pages to print for each signature in traditional bookbinding.

The input PDF is already imposed (2 content pages per sheet side).
Each signature contains 16 content pages = 8 PDF pages (4 sheets × 2 sides).

For duplex printing with even pages in reverse:
  - First pass: print odd PDF pages (1, 3, 5, 7)
  - Flip the stack
  - Second pass: print even PDF pages in reverse (8, 6, 4, 2)

The script automatically pads to the nearest multiple of 8 PDF pages.")]
struct Cli {
    /// Total number of pages in the imposed PDF
    #[arg(value_name = "TOTAL_PAGES")]
    total_pages: String,
}

fn main() {
    let args = Cli::parse();

    // Parse the total pages argument
    let total_pages: usize = match args.total_pages.parse() {
        Ok(n) if n > 0 => n,
        Ok(_) => {
            eprintln!("Error: Invalid page count - Page count must be positive");
            process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: Invalid page count - {e}");
            process::exit(1);
        }
    };

    // Print the signature information
    print_signatures(total_pages);
}
