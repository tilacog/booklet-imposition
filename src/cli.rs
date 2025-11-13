//! Command-line interface definitions

use crate::error::{PdfError, Result};
use clap::Parser;
use std::path::PathBuf;

/// Command-line arguments for the booklet tool
#[derive(Parser, Debug)]
#[command(name = "booklet")]
#[command(version, about = "Prepare a PDF for booklet printing")]
#[command(long_about = None)]
#[command(after_help = r"EXAMPLES:
    booklet input.pdf
    booklet input.pdf --paper letterpaper
    booklet input.pdf --paper a4paper --bleed 9
    booklet input.pdf -p letterpaper -b 9
    booklet input.pdf --signature 16              # 16-page signatures
    booklet input.pdf -s 32 -p a4paper            # 32-page signatures on A4

PAPER SIZES:
    Common values include: a4paper (default), letterpaper, a5paper, legalpaper

BLEED:
    Bleed amount in points (72 points = 1 inch, ~3 points = 1mm)
    Use positive values to add bleed around content (e.g., 9 for ~3mm bleed)

SIGNATURES:
    A signature (or section) is a group of folded pages in bookbinding.
    Instead of one large booklet, the PDF is split into multiple smaller
    signatures that are then gathered and bound together.
    Signature size must be divisible by 4. Common values: 8, 16, 32
    Use 0 (default) to treat the entire PDF as a single booklet.
")]
pub struct Cli {
    /// Path to input PDF file
    #[arg(value_name = "INPUT_PDF")]
    pub input: PathBuf,

    /// Paper size for booklet
    #[arg(short, long, default_value = "a4paper", value_name = "SIZE")]
    pub paper: String,

    /// Bleed amount in points
    #[arg(short, long, default_value_t = 0, value_name = "POINTS")]
    pub bleed: i32,

    /// Pages per signature/section (0 = single booklet)
    #[arg(short, long, default_value_t = 0, value_name = "PAGES")]
    pub signature: usize,
}

impl Cli {
    /// Validate the command-line arguments
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Input file does not exist
    /// - Bleed value is negative
    /// - Signature size is not divisible by 4 (when non-zero)
    pub fn validate(&self) -> Result<()> {
        // Check if input file exists
        if !self.input.exists() {
            return Err(PdfError::FileNotFound(self.input.clone()));
        }

        // Validate bleed is non-negative
        if self.bleed < 0 {
            return Err(PdfError::InvalidBleed(self.bleed));
        }

        // Validate signature size
        if self.signature > 0 && self.signature % 4 != 0 {
            return Err(PdfError::InvalidSignatureSize(self.signature));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_validate_missing_file() {
        let cli = Cli {
            input: PathBuf::from("/nonexistent/file.pdf"),
            paper: "a4paper".to_string(),
            bleed: 0,
            signature: 0,
        };

        let result = cli.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PdfError::FileNotFound(_)));
    }

    #[test]
    fn test_validate_negative_bleed() {
        let temp_dir = TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("test.pdf");
        File::create(&temp_file).unwrap();

        let cli = Cli {
            input: temp_file,
            paper: "a4paper".to_string(),
            bleed: -5,
            signature: 0,
        };

        let result = cli.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PdfError::InvalidBleed(-5)));
    }

    #[test]
    fn test_validate_invalid_signature_size() {
        let temp_dir = TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("test.pdf");
        File::create(&temp_file).unwrap();

        let cli = Cli {
            input: temp_file,
            paper: "a4paper".to_string(),
            bleed: 0,
            signature: 10, // Not divisible by 4
        };

        let result = cli.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PdfError::InvalidSignatureSize(10)
        ));
    }

    #[test]
    fn test_validate_valid_arguments() {
        let temp_dir = TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("test.pdf");
        File::create(&temp_file).unwrap();

        let cli = Cli {
            input: temp_file,
            paper: "a4paper".to_string(),
            bleed: 9,
            signature: 16,
        };

        let result = cli.validate();
        assert!(result.is_ok());
    }
}
