//! Transform Box, Vec and HashMap while recycling the backing memory if possible.
//!
//! <p style="font-family: 'Fira Sans',sans-serif;padding:0.3em 0"><strong>
//! <a href="https://crates.io/crates/recycle">📦&nbsp;&nbsp;Crates.io</a>&nbsp;&nbsp;│&nbsp;&nbsp;<a href="https://github.com/alecmocatta/recycle">📑&nbsp;&nbsp;GitHub</a>&nbsp;&nbsp;│&nbsp;&nbsp;<a href="https://constellation.zulipchat.com/#narrow/stream/213236-subprojects">💬&nbsp;&nbsp;Chat</a>
//! </strong></p>
//!
//!
//!
//! # Example
//!
//! ```text
//! ...
//! ```

#![doc(html_root_url = "https://docs.rs/recycle/0.1.0")]
#![warn(
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	trivial_casts,
	trivial_numeric_casts,
	unused_import_braces,
	unused_qualifications,
	unused_results,
	clippy::pedantic
)]
#![allow(clippy::module_name_repetitions, clippy::missing_errors_doc)]

mod r#try;
mod vec;

pub use self::{r#try::*, vec::*};

#[cfg(test)]
mod tests {
	// use super::*;

	#[test]
	fn succeeds() {}
}
