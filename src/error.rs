//! Error types for PDF booklet imposition operations

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias using our custom error type
pub type Result<T> = std::result::Result<T, PdfError>;

/// Error types that can occur during PDF booklet processing
#[derive(Error, Debug)]
pub enum PdfError {
    /// A required external dependency is missing
    #[error("Missing dependency: {0}")]
    MissingDependency(String),

    /// Input file was not found
    #[error("File not found: {}", .0.display())]
    FileNotFound(PathBuf),

    /// Invalid page count returned from PDF
    #[error("Invalid page count: {0}")]
    InvalidPageCount(String),

    /// External subprocess failed
    #[error("Subprocess failed: {0}")]
    SubprocessFailed(String),

    /// Invalid bleed value
    #[error("Bleed must be non-negative (got: {0})")]
    InvalidBleed(i32),

    /// Invalid signature size
    #[error("Signature size ({0}) must be divisible by 4")]
    InvalidSignatureSize(usize),

    /// Negative signature size
    #[error("Signature size must be non-negative (got: {0})")]
    NegativeSignatureSize(i32),

    /// Output file was not created
    #[error("Failed to create output file: {}", .0.display())]
    OutputNotCreated(PathBuf),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Page count is not divisible by signature size
    #[error("Total pages ({0}) is not divisible by signature size ({1})")]
    PageCountMismatch(usize, usize),
}
