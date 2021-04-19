use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    ops::Mul,
};

/// An ordered array of stats that an item can have (q.v. `ItemState`), or that
/// a scroll can grant on success (q.v. `Scroll`).
///
/// The elements of the stat array are not identified nominally, and can only
/// be identified by their position/index within the array. As such, obviously,
/// care must be taken when inserting elements into, and when copying to/from,
/// the stats array. Particularly, the exact indices of the elements in a given
/// stat array must be respected.
///
/// When performing operations that take two different `Stats` structs, like
/// testing for equality, `Stats::max_in_place`, etc., it is assumed that the
/// two `Stats` structs have stat arrays of equal length. This invariant is
/// only checked using debug assertions; in release mode, breaking this
/// invariant might still panic, or might even silently fail!
#[derive(Clone, Debug)]
pub struct Stats {
    stats: Vec<u16>,
}

impl Stats {
    /// Creates a new `Stats` using the provided stat array.
    pub const fn from_vec(stats: Vec<u16>) -> Self {
        Self { stats }
    }

    /// The length of this `Stats`'s stat array.
    pub fn len(&self) -> usize {
        self.stats.len()
    }

    /// Adds `self` to `other`, using ordinary addition, and returns the result
    /// as a freshly-allocated `Stats`.
    ///
    /// ## Invariants:
    ///
    /// - `self.stats.len() == other.stats.len()`
    pub fn plus(&self, other: &Self) -> Self {
        debug_assert_eq!(self.stats.len(), other.stats.len());

        Self {
            stats: self
                .stats
                .iter()
                .zip(other.stats.iter())
                .map(|(s0, s1)| s0 + s1)
                .collect(),
        }
    }

    /// Performs summation of `self` with `other`, over the [max
    /// tropical](https://en.wikipedia.org/wiki/Tropical_semiring)
    /// [semimodule](https://en.wikipedia.org/wiki/Semimodule), but the result
    /// is simply used to mutate `self` in-place.
    ///
    /// This function is used by `Scroll::master_scroll`.
    ///
    /// ## Invariants:
    ///
    /// - `self.stats.len() == other.stats.len()`
    pub fn max_in_place(&mut self, other: &Stats) {
        debug_assert_eq!(self.stats.len(), other.stats.len());

        for (i, stat) in other.stats.iter().enumerate() {
            if stat > &self.stats[i] {
                self.stats[i] = *stat;
            }
        }
    }
}

impl PartialEq for Stats {
    /// ## Invariants:
    ///
    /// - `self.stats.len() == other.stats.len()`
    fn eq(&self, other: &Self) -> bool {
        debug_assert_eq!(self.stats.len(), other.stats.len());

        self.stats == other.stats
    }
}

impl Eq for Stats {}

impl Hash for Stats {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.stats.hash(state);
    }
}

impl PartialOrd for Stats {
    /// Returns `None` any time that `self.stats.len() != other.stats.len()`.
    /// Returns `None` any time that a particular stat is _larger_ in `self`
    /// (compared to the same stat in `other`), but another particular stat is
    /// _smaller_ in `self`.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.stats.len() != other.stats.len() {
            return None;
        }

        let mut part_ord = None;

        for ord in self
            .stats
            .iter()
            .zip(other.stats.iter())
            .map(|(s0, s1)| s0.cmp(s1))
        {
            match part_ord {
                None => part_ord = Some(ord),
                Some(Ordering::Less) => {
                    if ord == Ordering::Greater {
                        return None;
                    }
                }
                Some(Ordering::Equal) => {
                    if ord != Ordering::Equal {
                        part_ord = Some(ord);
                    }
                }
                Some(Ordering::Greater) => {
                    if ord == Ordering::Less {
                        return None;
                    }
                }
            }
        }

        part_ord
    }
}

impl Mul<u16> for Stats {
    type Output = Stats;

    /// Implements [scalar
    /// multiplication](https://en.wikipedia.org/wiki/Scalar_multiplication).
    fn mul(mut self, rhs: u16) -> Self::Output {
        for stat in self.stats.iter_mut() {
            *stat *= rhs;
        }

        self
    }
}
