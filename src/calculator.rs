//! Calculate PDF page numbers to print for signature printing in traditional bookbinding.
//!
//! The input PDF is already imposed (2 content pages per sheet side).
//! Each signature contains 16 content pages = 8 PDF pages (4 sheets × 2 sides).
//!
//! For printing even-numbered PDF pages in reverse, you print:
//! - First pass: odd PDF pages (1, 3, 5, 7)
//! - Flip the stack
//! - Second pass: even PDF pages in reverse (8, 6, 4, 2)

/// Represents a single signature with its page numbers for duplex printing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    /// Signature number (1-indexed)
    pub number: usize,
    /// Odd PDF pages to print in first pass
    pub odd_pages: Vec<usize>,
    /// Even PDF pages to print in second pass (in reverse order)
    pub even_pages: Vec<usize>,
}

impl Signature {
    /// Format the signature for display with ordinal suffix
    #[must_use] 
    pub fn ordinal_suffix(&self) -> &'static str {
        match self.number {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    }
}

/// Calculate which PDF pages to print for each signature.
///
/// # Arguments
///
/// * `total_pdf_pages` - Total number of pages in the imposed PDF
///
/// # Returns
///
/// A tuple of (signatures vector, padded page count)
///
/// # Examples
///
/// ```
/// use booklet_imposition::calculator::calculate_signatures;
///
/// let (signatures, padded) = calculate_signatures(8);
/// assert_eq!(signatures.len(), 1);
/// assert_eq!(padded, 8);
/// assert_eq!(signatures[0].odd_pages, vec![1, 3, 5, 7]);
/// assert_eq!(signatures[0].even_pages, vec![8, 6, 4, 2]);
/// ```
#[must_use] 
pub fn calculate_signatures(total_pdf_pages: usize) -> (Vec<Signature>, usize) {
    const PDF_PAGES_PER_SIGNATURE: usize = 8; // 4 sheets × 2 sides

    // Pad to nearest multiple of 8
    let padded_pages = total_pdf_pages.div_ceil(PDF_PAGES_PER_SIGNATURE)
        * PDF_PAGES_PER_SIGNATURE;

    let num_signatures = padded_pages / PDF_PAGES_PER_SIGNATURE;
    let mut signatures = Vec::with_capacity(num_signatures);

    for sig_num in 0..num_signatures {
        let start_pdf_page = sig_num * PDF_PAGES_PER_SIGNATURE + 1;
        let end_pdf_page = start_pdf_page + PDF_PAGES_PER_SIGNATURE - 1;

        // First pass: odd PDF pages in order
        let odd_pages: Vec<usize> = (start_pdf_page..=end_pdf_page).step_by(2).collect();

        // Second pass: even PDF pages in reverse
        let even_pages: Vec<usize> = (start_pdf_page..=end_pdf_page)
            .rev()
            .filter(|p| p % 2 == 0)
            .collect();

        signatures.push(Signature {
            number: sig_num + 1,
            odd_pages,
            even_pages,
        });
    }

    (signatures, padded_pages)
}

/// Print formatted signature information to stdout
pub fn print_signatures(total_pdf_pages: usize) {
    let (signatures, padded_pages) = calculate_signatures(total_pdf_pages);

    println!("Total PDF pages: {total_pdf_pages}");
    if padded_pages != total_pdf_pages {
        println!(
            "Padded to: {} pages ({} blank PDF pages needed)",
            padded_pages,
            padded_pages - total_pdf_pages
        );
    }
    println!("Number of signatures: {}", signatures.len());
    println!();

    for sig in &signatures {
        println!(
            "{}{} signature:",
            sig.number,
            sig.ordinal_suffix()
        );
        println!(
            "  - {}",
            sig.odd_pages
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!(
            "  - {}",
            sig.even_pages
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_signature_exact() {
        let (signatures, padded) = calculate_signatures(8);

        assert_eq!(signatures.len(), 1);
        assert_eq!(padded, 8);

        let sig = &signatures[0];
        assert_eq!(sig.number, 1);
        assert_eq!(sig.odd_pages, vec![1, 3, 5, 7]);
        assert_eq!(sig.even_pages, vec![8, 6, 4, 2]);
    }

    #[test]
    fn test_multiple_signatures_exact() {
        let (signatures, padded) = calculate_signatures(24);

        assert_eq!(signatures.len(), 3);
        assert_eq!(padded, 24);

        // First signature
        assert_eq!(signatures[0].odd_pages, vec![1, 3, 5, 7]);
        assert_eq!(signatures[0].even_pages, vec![8, 6, 4, 2]);

        // Second signature
        assert_eq!(signatures[1].odd_pages, vec![9, 11, 13, 15]);
        assert_eq!(signatures[1].even_pages, vec![16, 14, 12, 10]);

        // Third signature
        assert_eq!(signatures[2].odd_pages, vec![17, 19, 21, 23]);
        assert_eq!(signatures[2].even_pages, vec![24, 22, 20, 18]);
    }

    #[test]
    fn test_padding_needed() {
        let (signatures, padded) = calculate_signatures(10);

        assert_eq!(signatures.len(), 2);
        assert_eq!(padded, 16); // Padded to next multiple of 8

        // First signature
        assert_eq!(signatures[0].odd_pages, vec![1, 3, 5, 7]);
        assert_eq!(signatures[0].even_pages, vec![8, 6, 4, 2]);

        // Second signature (pages 9-10 exist, 11-16 are blanks)
        assert_eq!(signatures[1].odd_pages, vec![9, 11, 13, 15]);
        assert_eq!(signatures[1].even_pages, vec![16, 14, 12, 10]);
    }

    #[test]
    fn test_single_page() {
        let (signatures, padded) = calculate_signatures(1);

        assert_eq!(signatures.len(), 1);
        assert_eq!(padded, 8);

        assert_eq!(signatures[0].odd_pages, vec![1, 3, 5, 7]);
        assert_eq!(signatures[0].even_pages, vec![8, 6, 4, 2]);
    }

    #[test]
    fn test_boundary_case_7_pages() {
        let (signatures, padded) = calculate_signatures(7);

        assert_eq!(signatures.len(), 1);
        assert_eq!(padded, 8);

        assert_eq!(signatures[0].odd_pages, vec![1, 3, 5, 7]);
        assert_eq!(signatures[0].even_pages, vec![8, 6, 4, 2]);
    }

    #[test]
    fn test_boundary_case_9_pages() {
        let (signatures, padded) = calculate_signatures(9);

        assert_eq!(signatures.len(), 2);
        assert_eq!(padded, 16);
    }

    #[test]
    fn test_large_document() {
        let (signatures, padded) = calculate_signatures(100);

        let expected_signatures = 13; // 100 / 8 = 12.5, rounds up to 13
        assert_eq!(signatures.len(), expected_signatures);
        assert_eq!(padded, 104); // 13 * 8

        // Check last signature
        let last_sig = &signatures[signatures.len() - 1];
        assert_eq!(last_sig.number, 13);
        assert_eq!(last_sig.odd_pages, vec![97, 99, 101, 103]);
        assert_eq!(last_sig.even_pages, vec![104, 102, 100, 98]);
    }

    #[test]
    fn test_signature_numbering() {
        let (signatures, _) = calculate_signatures(24);

        for (i, sig) in signatures.iter().enumerate() {
            assert_eq!(sig.number, i + 1);
        }
    }

    #[test]
    fn test_even_pages_reversed() {
        let (signatures, _) = calculate_signatures(16);

        for sig in &signatures {
            let even = &sig.even_pages;
            // Check descending order
            for i in 0..(even.len() - 1) {
                assert!(even[i] > even[i + 1]);
            }
        }
    }

    #[test]
    fn test_odd_pages_ascending() {
        let (signatures, _) = calculate_signatures(16);

        for sig in &signatures {
            let odd = &sig.odd_pages;
            // Check ascending order
            for i in 0..(odd.len() - 1) {
                assert!(odd[i] < odd[i + 1]);
            }
        }
    }

    #[test]
    fn test_pages_per_signature() {
        let (signatures, _) = calculate_signatures(32);

        for sig in &signatures {
            assert_eq!(sig.odd_pages.len(), 4);
            assert_eq!(sig.even_pages.len(), 4);
        }
    }

    #[test]
    fn test_no_duplicate_pages() {
        let (signatures, _) = calculate_signatures(24);

        let mut all_pages: Vec<usize> = Vec::new();
        for sig in &signatures {
            all_pages.extend(&sig.odd_pages);
            all_pages.extend(&sig.even_pages);
        }

        // Check no duplicates by comparing length with unique count
        let mut sorted_pages = all_pages.clone();
        sorted_pages.sort_unstable();
        sorted_pages.dedup();
        assert_eq!(all_pages.len(), sorted_pages.len());
    }

    #[test]
    fn test_continuous_page_range() {
        let (signatures, padded) = calculate_signatures(20);

        let mut all_pages: Vec<usize> = Vec::new();
        for sig in &signatures {
            all_pages.extend(&sig.odd_pages);
            all_pages.extend(&sig.even_pages);
        }

        all_pages.sort_unstable();
        let expected_pages: Vec<usize> = (1..=padded).collect();

        assert_eq!(all_pages, expected_pages);
    }

    #[test]
    fn test_ordinal_suffix() {
        let sig1 = Signature {
            number: 1,
            odd_pages: vec![],
            even_pages: vec![],
        };
        let sig2 = Signature {
            number: 2,
            odd_pages: vec![],
            even_pages: vec![],
        };
        let sig3 = Signature {
            number: 3,
            odd_pages: vec![],
            even_pages: vec![],
        };
        let sig4 = Signature {
            number: 4,
            odd_pages: vec![],
            even_pages: vec![],
        };

        assert_eq!(sig1.ordinal_suffix(), "st");
        assert_eq!(sig2.ordinal_suffix(), "nd");
        assert_eq!(sig3.ordinal_suffix(), "rd");
        assert_eq!(sig4.ordinal_suffix(), "th");
    }
}
