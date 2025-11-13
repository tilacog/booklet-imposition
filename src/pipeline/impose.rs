//! Phase 4: Booklet imposition using pdfbook2

use crate::error::{PdfError, Result};
use crate::process::{check_dependency, detect_pdfbook2_version, Pdfbook2Version};
use std::path::Path;
use std::process::Command;

/// Transform a padded PDF into booklet format using pdfbook2.
///
/// This function rearranges pages for duplex (double-sided) booklet printing.
/// Pages are reordered so that when printed on both sides and folded, they
/// appear in the correct sequence.
///
/// # Arguments
///
/// * `input_path` - Path to the padded PDF file (must have page count divisible by 4)
/// * `output_path` - Path where the booklet PDF will be saved
/// * `paper_size` - Paper size specification (e.g., 'a4paper', 'letterpaper')
///
/// # Errors
///
/// Returns an error if:
/// - `pdfbook2` command is not available
/// - `pdfbook2` version detection fails
/// - `pdfbook2` subprocess fails
/// - Output file is not generated after `pdfbook2` execution
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use booklet_imposition::pipeline::impose_booklet;
///
/// # fn main() -> booklet_imposition::Result<()> {
/// impose_booklet(
///     Path::new("input.pdf"),
///     Path::new("output.pdf"),
///     "a4paper"
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn impose_booklet(input_path: &Path, output_path: &Path, paper_size: &str) -> Result<()> {
    check_dependency("pdfbook2")?;

    println!("Creating booklet...");

    // Detect pdfbook2 version
    let version = detect_pdfbook2_version()?;

    match version {
        Pdfbook2Version::Modern => {
            // Modern pdfbook2: explicitly specify output file
            let output = Command::new("pdfbook2")
                .arg("--paper")
                .arg(paper_size)
                .arg("--no-crop")
                .arg(input_path)
                .arg("--outfile")
                .arg(output_path)
                .output()
                .map_err(|e| {
                    PdfError::SubprocessFailed(format!("Failed to run pdfbook2: {e}"))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(PdfError::SubprocessFailed(format!(
                    "pdfbook2 failed: {stderr}"
                )));
            }
        }
        Pdfbook2Version::Legacy => {
            // Legacy pdfbook2: auto-generates output filename
            let output = Command::new("pdfbook2")
                .arg("--paper")
                .arg(paper_size)
                .arg("--no-crop")
                .arg(input_path)
                .output()
                .map_err(|e| {
                    PdfError::SubprocessFailed(format!("Failed to run pdfbook2: {e}"))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(PdfError::SubprocessFailed(format!(
                    "pdfbook2 failed: {stderr}"
                )));
            }

            // Handle legacy pdfbook2 which writes <stem>-book.pdf
            let input_dir = input_path.parent().unwrap_or_else(|| Path::new("."));
            let input_stem = input_path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| {
                    PdfError::SubprocessFailed("Invalid input filename".to_string())
                })?;

            let candidate = input_dir.join(format!("{input_stem}-book.pdf"));

            if candidate.exists() {
                std::fs::rename(&candidate, output_path)?;
            }
        }
    }

    // Verify the booklet was created successfully
    if output_path.exists() {
        println!("Booklet created: {}", output_path.display());
        Ok(())
    } else {
        Err(PdfError::OutputNotCreated(output_path.to_path_buf()))
    }
}
