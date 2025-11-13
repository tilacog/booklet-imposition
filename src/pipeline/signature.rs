//! Phase 3 & 5: Split into signatures and merge them back

use crate::error::{PdfError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Split a PDF into multiple signature PDFs.
///
/// Each signature will contain exactly `signature_size` pages.
///
/// # Arguments
///
/// * `input_path` - Path to the input PDF file
/// * `output_dir` - Directory where signature PDFs will be saved
/// * `signature_size` - Number of pages per signature
/// * `total_pages` - Total number of pages in the input PDF
///
/// # Returns
///
/// List of paths to the created signature PDFs
///
/// # Errors
///
/// Returns an error if:
/// - Total pages is not divisible by signature size
/// - Directory creation fails
/// - `ghostscript` subprocess fails during extraction
/// - Output file is not generated after extraction
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use booklet_imposition::pipeline::split_into_signatures;
///
/// # fn main() -> booklet_imposition::Result<()> {
/// let signatures = split_into_signatures(
///     Path::new("input.pdf"),
///     Path::new("signatures"),
///     16,
///     32
/// )?;
/// println!("Created {} signatures", signatures.len());
/// # Ok(())
/// # }
/// ```
pub fn split_into_signatures(
    input_path: &Path,
    output_dir: &Path,
    signature_size: usize,
    total_pages: usize,
) -> Result<Vec<PathBuf>> {
    if total_pages % signature_size != 0 {
        return Err(PdfError::PageCountMismatch(total_pages, signature_size));
    }

    let num_signatures = total_pages / signature_size;

    println!(
        "Splitting into {num_signatures} signature(s) of {signature_size} pages each..."
    );

    let mut signature_paths = Vec::with_capacity(num_signatures);

    for sig_num in 0..num_signatures {
        let start_page = sig_num * signature_size + 1;
        let end_page = start_page + signature_size - 1;

        let output_path = output_dir.join(format!("signature_{:03}.pdf", sig_num + 1));

        // Use ghostscript to extract page range
        let output = Command::new("gs")
            .arg("-q")
            .arg("-dNOPAUSE")
            .arg("-dBATCH")
            .arg("-sDEVICE=pdfwrite")
            .arg(format!("-dFirstPage={start_page}"))
            .arg(format!("-dLastPage={end_page}"))
            .arg(format!("-sOutputFile={}", output_path.display()))
            .arg(input_path)
            .output()
            .map_err(|e| {
                PdfError::SubprocessFailed(format!(
                    "Failed to extract signature {}: {}",
                    sig_num + 1,
                    e
                ))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PdfError::SubprocessFailed(format!(
                "Failed to extract signature {}: {}",
                sig_num + 1,
                stderr
            )));
        }

        if !output_path.exists() {
            return Err(PdfError::OutputNotCreated(output_path.clone()));
        }

        signature_paths.push(output_path);
        println!(
            "  Created signature {}/{}: pages {}-{}",
            sig_num + 1,
            num_signatures,
            start_page,
            end_page
        );
    }

    Ok(signature_paths)
}

/// Merge multiple imposed signature PDFs into a single PDF.
///
/// # Arguments
///
/// * `signature_paths` - List of paths to imposed signature PDFs
/// * `output_path` - Path where the merged PDF will be saved
///
/// # Errors
///
/// Returns an error if:
/// - `ghostscript` subprocess fails during merging
/// - Output file is not generated after merging
///
/// # Examples
///
/// ```no_run
/// use std::path::{Path, PathBuf};
/// use booklet_imposition::pipeline::merge_signatures;
///
/// # fn main() -> booklet_imposition::Result<()> {
/// let signatures = vec![
///     PathBuf::from("sig1.pdf"),
///     PathBuf::from("sig2.pdf"),
/// ];
/// merge_signatures(&signatures, Path::new("merged.pdf"))?;
/// # Ok(())
/// # }
/// ```
pub fn merge_signatures(signature_paths: &[PathBuf], output_path: &Path) -> Result<()> {
    println!(
        "Merging {} signature(s) into final booklet...",
        signature_paths.len()
    );

    // Build ghostscript command
    let mut cmd = Command::new("gs");
    cmd.arg("-q")
        .arg("-dNOPAUSE")
        .arg("-dBATCH")
        .arg("-sDEVICE=pdfwrite")
        .arg(format!("-sOutputFile={}", output_path.display()));

    // Add all signature paths
    for sig_path in signature_paths {
        cmd.arg(sig_path);
    }

    let output = cmd.output().map_err(|e| {
        PdfError::SubprocessFailed(format!("Failed to run ghostscript for merging: {e}"))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PdfError::SubprocessFailed(format!(
            "Failed to merge signatures: {stderr}"
        )));
    }

    if !output_path.exists() {
        return Err(PdfError::OutputNotCreated(output_path.to_path_buf()));
    }

    println!("Signatures merged: {}", output_path.display());

    Ok(())
}
