//! Pipeline modules for PDF booklet processing

pub mod bleed;
pub mod impose;
pub mod pad;
pub mod signature;

// Re-export main functions
pub use bleed::add_bleed;
pub use impose::impose_booklet;
pub use pad::pad_pdf;
pub use signature::{merge_signatures, split_into_signatures};
