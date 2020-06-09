//! Main entry point for GozScoring

#![deny(warnings, missing_docs, trivial_casts, unused_qualifications)]
#![forbid(unsafe_code)]

use goz_scoring::application::APPLICATION;

/// Boot GozScoring
fn main() {
    abscissa_core::boot(&APPLICATION);
}
