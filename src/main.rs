//! PDF Booklet Imposition CLI
//!
//! Transform PDFs into printable booklet format with optional bleed and signature support.

use booklet_imposition::{
    pipeline::{add_bleed, impose_booklet, merge_signatures, pad_pdf, split_into_signatures},
    process::check_dependency,
    Cli, Result,
};
use clap::Parser;
use std::path::PathBuf;
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("ERROR: {e}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    // Parse command-line arguments
    let args = Cli::parse();

    // Validate arguments
    args.validate()?;

    // Check for required system dependencies
    check_dependency("gs")?;
    check_dependency("pdfbook2")?;
    if args.bleed > 0 {
        check_dependency("pdfcrop")?;
    }

    // Generate output filenames based on input
    let pdf_path = &args.input;
    let pdf_dir = pdf_path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let pdf_stem = pdf_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            booklet_imposition::PdfError::SubprocessFailed("Invalid input filename".to_string())
        })?;

    let bleed_pdf = pdf_dir.join(format!("{pdf_stem}_bleed.pdf"));
    let padded_pdf = pdf_dir.join(format!("{pdf_stem}_padded.pdf"));
    let booklet_pdf = pdf_dir.join(format!("{pdf_stem}_booklet.pdf"));

    // Phase 1: Add bleed to the PDF
    println!("=== Phase 1: Adding bleed ===");
    add_bleed(pdf_path, &bleed_pdf, args.bleed)?;
    println!();

    // Phase 2: Pad the PDF to appropriate page count
    println!("=== Phase 2: Padding PDF ===");
    let total = pad_pdf(&bleed_pdf, &padded_pdf, args.signature)?;
    println!("Total pages after padding: {total}");
    println!();

    // Choose workflow based on whether signatures are requested
    if args.signature > 0 {
        // Signature-based workflow
        println!(
            "=== Phase 3: Splitting into signatures ({} pages each) ===",
            args.signature
        );

        // Create temporary directory for signature files
        let sig_dir = pdf_dir.join(format!("{pdf_stem}_signatures"));
        std::fs::create_dir_all(&sig_dir)?;

        // Split into signatures
        let signature_paths = split_into_signatures(&padded_pdf, &sig_dir, args.signature, total)?;
        println!();

        // Phase 4: Impose each signature
        println!("=== Phase 4: Imposing signatures ===");
        let mut imposed_paths = Vec::with_capacity(signature_paths.len());

        for (i, sig_path) in signature_paths.iter().enumerate() {
            let sig_filename = sig_path.file_name().unwrap();
            let imposed_path = sig_dir.join(format!("imposed_{}", sig_filename.to_string_lossy()));

            println!(
                "Imposing signature {}/{}...",
                i + 1,
                signature_paths.len()
            );
            impose_booklet(sig_path, &imposed_path, &args.paper)?;

            imposed_paths.push(imposed_path);
        }
        println!();

        // Phase 5: Merge imposed signatures
        println!("=== Phase 5: Merging signatures ===");
        merge_signatures(&imposed_paths, &booklet_pdf)?;
        println!();

        // Clean up signature files
        println!("Cleaning up signature files...");
        std::fs::remove_dir_all(&sig_dir)?;
    } else {
        // Single booklet workflow (original behavior)
        println!("=== Phase 3: Creating booklet ===");
        impose_booklet(&padded_pdf, &booklet_pdf, &args.paper)?;
        println!();
    }

    // Clean up intermediate files
    cleanup_file(&bleed_pdf)?;
    cleanup_file(&padded_pdf)?;

    // Display final results
    println!();
    println!("Done.");
    println!("  Booklet: {}", booklet_pdf.display());
    if args.signature > 0 {
        let num_signatures = total / args.signature;
        println!(
            "  Signatures: {} × {} pages",
            num_signatures, args.signature
        );
    }

    Ok(())
}

fn cleanup_file(path: &PathBuf) -> Result<()> {
    if path.exists() {
        std::fs::remove_file(path)?;
        println!("Cleaned up intermediate file: {}", path.display());
    }
    Ok(())
}

