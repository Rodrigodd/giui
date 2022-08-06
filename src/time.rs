//! mod used for aliasing time when necessary.

pub use std::time::Duration;

#[cfg(not(test))]
pub use instant::Instant;
#[cfg(test)]
pub use mock_instant::Instant;
