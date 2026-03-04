//! Generic weight types for graph edges.
//!
//! The [`Weight`] trait abstracts over numeric types used as edge weights.
//! Supported out of the box: `f64`, `f32`, `u32`, `u64`.

use core::ops::Add;

/// Trait for weight types supported by the shortest path algorithms.
///
/// A weight must support:
/// - Copying (cheap, no heap allocation)
/// - Partial ordering (for comparing distances)
/// - Addition (for accumulating path costs)
/// - An infinity value (for unreachable nodes)
/// - A zero value (for the source node)
///
/// # Example
///
/// ```
/// use dmmsy::Weight;
///
/// // All standard numeric types implement Weight
/// assert_eq!(f64::ZERO, 0.0);
/// assert_eq!(f64::INFINITY, f64::INFINITY);
/// ```
pub trait Weight: Copy + PartialOrd + Add<Output = Self> + Default + core::fmt::Debug {
    /// A value greater than any valid weight. Used to initialize distances.
    const INFINITY: Self;

    /// The additive identity. Distance from source to itself.
    const ZERO: Self;

    /// Returns true if this weight is infinite (unreachable).
    fn is_infinite(self) -> bool;

    /// Convert to f64. Used for computing mean edge weight.
    fn to_f64(self) -> f64;
}

impl Weight for f64 {
    const INFINITY: f64 = f64::INFINITY;
    const ZERO: f64 = 0.0;

    #[inline]
    fn is_infinite(self) -> bool { self == f64::INFINITY }
    #[inline]
    fn to_f64(self) -> f64 { self }
}

impl Weight for f32 {
    const INFINITY: f32 = f32::INFINITY;
    const ZERO: f32 = 0.0;

    #[inline]
    fn is_infinite(self) -> bool { self == f32::INFINITY }
    #[inline]
    fn to_f64(self) -> f64 { self as f64 }
}

impl Weight for u32 {
    const INFINITY: u32 = u32::MAX;
    const ZERO: u32 = 0;

    #[inline]
    fn is_infinite(self) -> bool { self == u32::MAX }
    #[inline]
    fn to_f64(self) -> f64 { self as f64 }
}

impl Weight for u64 {
    const INFINITY: u64 = u64::MAX;
    const ZERO: u64 = 0;

    #[inline]
    fn is_infinite(self) -> bool { self == u64::MAX }
    #[inline]
    fn to_f64(self) -> f64 { self as f64 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_plus_zero_is_zero() {
        assert_eq!(f64::ZERO + f64::ZERO, f64::ZERO);
        assert_eq!(u32::ZERO + u32::ZERO, u32::ZERO);
    }

    #[test]
    fn infinity_is_infinite() {
        assert!(f64::INFINITY.is_infinite());
        assert!(f32::INFINITY.is_infinite());
        assert!(u32::INFINITY.is_infinite());
        assert!(u64::INFINITY.is_infinite());
    }

    #[test]
    fn zero_is_not_infinite() {
        assert!(!f64::ZERO.is_infinite());
        assert!(!u32::ZERO.is_infinite());
    }

    #[test]
    fn ordering_works() {
        assert!(f64::ZERO < f64::INFINITY);
        assert!(u32::ZERO < u32::INFINITY);
        let a: f64 = 1.0;
        let b: f64 = 2.0;
        assert!(a < b);
    }
}
