//! Phase 2: Pad PDF to ensure page count is appropriate for binding

use crate::error::{PdfError, Result};
use std::path::Path;
use std::process::Command;

/// Pad a PDF file to ensure total pages is a multiple of 4 (or `signature_size` if specified).
///
/// Booklet printing requires a page count divisible by 4 (each sheet has 4 pages
/// when folded: front-left, front-right, back-left, back-right). When using signatures,
/// the page count must be divisible by `signature_size`.
///
/// # Arguments
///
/// * `input_path` - Path to the input PDF file
/// * `output_path` - Path where the padded PDF will be saved
/// * `signature_size` - Pages per signature (0 = treat as single booklet, pad to multiple of 4)
///
/// # Returns
///
/// Total number of pages after padding
///
/// # Errors
///
/// Returns an error if:
/// - `ghostscript` subprocess fails
/// - Page count cannot be parsed from ghostscript output
/// - Padding operation fails (when adding blank pages)
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use booklet_imposition::pipeline::pad_pdf;
///
/// # fn main() -> booklet_imposition::Result<()> {
/// let total_pages = pad_pdf(
///     Path::new("input.pdf"),
///     Path::new("output.pdf"),
///     0 // Single booklet mode
/// )?;
/// println!("Total pages after padding: {}", total_pages);
/// # Ok(())
/// # }
/// ```
pub fn pad_pdf(input_path: &Path, output_path: &Path, signature_size: usize) -> Result<usize> {
    println!("Counting pages in input PDF...");

    // Count pages using ghostscript
    // Note: -dNOSAFER is required to read files, but we only read user-provided input
    let gs_cmd = format!(
        "({}) (r) file runpdfbegin pdfpagecount = quit",
        input_path.display()
    );

    let output = Command::new("gs")
        .args(["-q", "-dNOSAFER", "-dNODISPLAY", "-c", &gs_cmd])
        .output()
        .map_err(|e| {
            PdfError::SubprocessFailed(format!("Failed to run ghostscript: {e}"))
        })?;

    if !output.status.success() {
        return Err(PdfError::InvalidPageCount(
            "Could not determine page count from PDF".to_string(),
        ));
    }

    let total_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Validate the page count is a number
    let total: usize = total_str.parse().map_err(|_| {
        PdfError::InvalidPageCount(format!(
            "Could not parse page count: {total_str}"
        ))
    })?;

    // Calculate how many blank pages are needed
    let to_add = if signature_size > 0 {
        // When using signatures, each signature must have exactly signature_size pages
        (signature_size - (total % signature_size)) % signature_size
    } else {
        // Formula: (4 - (total % 4)) % 4 returns 0-3 blank pages needed
        (4 - (total % 4)) % 4
    };

    println!("Original pages: {total}");

    if to_add == 0 {
        println!("Multiple of 4 already - no padding required.");
        std::fs::copy(input_path, output_path)?;
    } else {
        println!("Adding {to_add} blank page(s)...");

        // Use ghostscript to append blank pages
        // We read the original PDF and then add blank pages using PostScript commands
        let blank_pages = "<</PageSize[595 842]>> setpagedevice showpage ".repeat(to_add);

        let output = Command::new("gs")
            .arg("-q")
            .arg("-dNOPAUSE")
            .arg("-dBATCH")
            .arg("-sDEVICE=pdfwrite")
            .arg(format!("-sOutputFile={}", output_path.display()))
            .arg(input_path)
            .arg("-c")
            .arg(&blank_pages)
            .output()
            .map_err(|e| {
                PdfError::SubprocessFailed(format!("Failed to run ghostscript for padding: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PdfError::SubprocessFailed(format!(
                "Failed to create padded PDF: {stderr}"
            )));
        }

        if !output_path.exists() {
            return Err(PdfError::OutputNotCreated(output_path.to_path_buf()));
        }
    }

    let new_total = total + to_add;
    println!("Padded PDF: {} ({} pages)", output_path.display(), new_total);

    Ok(new_total)
}
