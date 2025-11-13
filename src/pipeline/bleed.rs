//! Phase 1: Add bleed to PDF using pdfcrop

use crate::error::{PdfError, Result};
use crate::process::check_dependency;
use std::path::Path;
use std::process::Command;

/// Add bleed to a PDF using pdfcrop with positive margins.
///
/// Bleed is extra content area that extends beyond the final trim size,
/// ensuring no white edges appear after cutting.
///
/// # Arguments
///
/// * `input_path` - Path to the input PDF file
/// * `output_path` - Path where the PDF with bleed will be saved
/// * `bleed_amount` - Bleed in points (72 points = 1 inch, ~3pt = 1mm)
///
/// # Errors
///
/// Returns an error if:
/// - Bleed amount is negative
/// - `pdfcrop` command is not available
/// - File copy fails (when bleed is 0)
/// - `pdfcrop` subprocess fails
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use booklet_imposition::pipeline::add_bleed;
///
/// # fn main() -> booklet_imposition::Result<()> {
/// add_bleed(
///     Path::new("input.pdf"),
///     Path::new("output.pdf"),
///     9 // 9pt bleed (~3mm)
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn add_bleed(input_path: &Path, output_path: &Path, bleed_amount: i32) -> Result<()> {
    if bleed_amount < 0 {
        return Err(PdfError::InvalidBleed(bleed_amount));
    }

    if bleed_amount == 0 {
        println!("Bleed disabled (0pt) - copying file...");
        std::fs::copy(input_path, output_path)?;
        return Ok(());
    }

    check_dependency("pdfcrop")?;

    println!("Adding {bleed_amount}pt bleed to all sides...");

    // Positive margins add bleed by expanding the page
    let margins = format!(
        "{bleed_amount} {bleed_amount} {bleed_amount} {bleed_amount}"
    );

    let mut cmd = Command::new("pdfcrop");
    cmd.arg("--margins")
        .arg(&margins)
        .arg(input_path)
        .arg(output_path);

    // Run pdfcrop and suppress output
    let output = cmd
        .output()
        .map_err(|e| PdfError::SubprocessFailed(format!("Failed to run pdfcrop: {e}")))?;

    if !output.status.success() {
        return Err(PdfError::SubprocessFailed(
            "Failed to add bleed to PDF".to_string(),
        ));
    }

    // Verify output file was created
    if !output_path.exists() {
        return Err(PdfError::OutputNotCreated(output_path.to_path_buf()));
    }

    println!("Bleed added: {}", output_path.display());

    Ok(())
}
