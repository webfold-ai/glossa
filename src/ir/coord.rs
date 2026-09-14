//! Apeiron Coordinate and Invariant Hash Calculation.
//!
//! Grounded in the Apeiron basis: every translatable unit, regardless of surface
//! framework syntax (React, Vue, Svelte, Jinja, Fluent, PO), projects to an
//! invariant coordinate `ApeironCoord`.
//!
//! Uses Fibonacci hashing (`k * PHI`, golden ratio `0x9E3779B97F4A7C15`)
//! aligned with `hashtrinity`.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Golden ratio constant phi on 2^64: 2^64 / 1.61803398874989484820...
pub const PHI64: u64 = 0x9e3779b97f4a7c15;

/// Deterministic 64-bit coordinate identifying an Apeiron invariant term.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ApeironCoord(pub u64);

impl ApeironCoord {
    /// Compute the Apeiron coordinate from raw bytes via 64-bit Fibonacci fold.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut h: u64 = 0xcbf29ce484222325; // FNV-1a offset basis
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3); // FNV prime
        }
        // Multiply by PHI (Fibonacci bijection) for maximal dispersion
        Self(h.wrapping_mul(PHI64))
    }

    /// Compute coordinate from canonical string representation.
    pub fn from_str_canonical(s: &str) -> Self {
        Self::from_bytes(s.as_bytes())
    }

    /// Returns the raw 64-bit coordinate value.
    pub fn raw(&self) -> u64 {
        self.0
    }

    /// Hex representation of the coordinate.
    pub fn to_hex(&self) -> String {
        format!("{:016x}", self.0)
    }
}

impl fmt::Display for ApeironCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

impl fmt::Debug for ApeironCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ApeironCoord({:016x})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apeiron_coord_determinism() {
        let c1 = ApeironCoord::from_str_canonical("Hello, {0}!");
        let c2 = ApeironCoord::from_str_canonical("Hello, {0}!");
        assert_eq!(c1, c2);
        assert_ne!(c1, ApeironCoord::from_str_canonical("Hello, {1}!"));
    }
}
