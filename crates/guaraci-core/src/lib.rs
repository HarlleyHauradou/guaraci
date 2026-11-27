//! Guaraci core: physics, geometry, materials, sources and probes.
//! Nothing in this crate does I/O; engines (CPU, GPU) drive it.

pub mod electron;
pub mod geometry;
pub mod physics;
pub mod problem;
pub mod rng;
pub mod transport;
pub mod vec3;

/// Version, author and license, shown by `guaraci --version` and in every report.
pub const VERSION_LINE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " — © 2026 Harlley Hauradou — MIT OR Apache-2.0"
);
