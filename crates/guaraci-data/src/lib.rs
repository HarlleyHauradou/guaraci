//! Atomic and nuclear data for Guaraci.
//!
//! Libraries are never shipped with the code: they are downloaded from their
//! official publishers on first use, checked against a known SHA-256, converted
//! to a compact internal format and cached locally (see [`library`]).

pub mod decay;
pub mod electron;
pub mod elements;
pub mod endf;
pub mod library;
pub mod photon;

pub use decay::{NuclideDecay, PhotonKind, PhotonLine};
pub use elements::NuclideId;
pub use photon::{ElementPhoton, Table};

#[cfg(test)]
mod tests;
