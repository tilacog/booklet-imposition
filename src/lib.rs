//! PDF Booklet Imposition Library
//!
//! This library provides utilities for transforming PDFs into printable booklet format
//! with optional bleed and signature support for traditional bookbinding.

pub mod calculator;
pub mod cli;
pub mod error;
pub mod pipeline;
pub mod process;

// Re-export commonly used types
pub use calculator::{calculate_signatures, print_signatures, Signature};
pub use cli::Cli;
pub use error::{PdfError, Result};
