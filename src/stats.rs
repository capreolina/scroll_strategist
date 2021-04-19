use std::{cmp::Ordering, iter, ops::Mul};

/// An ordered array of stats that an item can have (q.v. `ItemState`), or that
/// a scroll can grant on success (q.v. `Scroll`).
///
/// The elements of the stat array are not identified nominally, and can only
/// be identified by their position/index within the array. As such, obviously,
/// care must be taken when inserting elements into, and when copying to/from,
/// the stats array. Particularly, the exact indices of the elements in a given
/// stat array must be respected.
///
/// A `Stats`'s stat array is treated as implicitly infinite in length, with
/// the "missing" elements implicitly being zeroes. As a result, any time that
/// the length of such an array matters (like, for example, when summing two
/// stat arrays together), some extra work is needed to treat any "missing"
/// elements properly.
#[derive(Clone, Debug, Default)]
pub struct Stats {
    stats: Vec<u16>,
}

impl Stats {
    /// Creates a new `Stats` using the provided stat array.
    pub const fn new_from_vec(stats: Vec<u16>) -> Self {
        Self { stats }
    }

    /// Adds `self` to `other`, using ordinary addition, and returns the result
    /// as a freshly-allocated `Stats`.
    pub fn plus(&self, other: &Self) -> Self {
        Self {
            stats: self
                .stats
                .iter()
                .chain(iter::repeat(&0))
                .zip(other.stats.iter().chain(iter::repeat(&0)))
                .map(|(s0, s1)| s0 + s1)
                .take(self.stats.len().max(other.stats.len()))
                .collect(),
        }
    }

    /// Performs summation of `self` with `other`, over the [max
    /// tropical](https://en.wikipedia.org/wiki/Tropical_semiring)
    /// [semimodule](https://en.wikipedia.org/wiki/Semimodule), but the result
    /// is simply used to mutate `self` in-place.
    ///
    /// This function is used by `Scroll::master_scroll`.
    pub fn max_in_place(&mut self, other: &Stats) {
        while other.stats.len() > self.stats.len() {
            self.stats.push(0);
        }

        for (i, stat) in other.stats.iter().enumerate() {
            if stat > &self.stats[i] {
                self.stats[i] = *stat;
            }
        }
    }
}

impl PartialEq for Stats {
    fn eq(&self, other: &Self) -> bool {
        self.stats
            .iter()
            .chain(iter::repeat(&0))
            .zip(other.stats.iter().chain(iter::repeat(&0)))
            .take(self.stats.len().max(other.stats.len()))
            .all(|(s0, s1)| s0 == s1)
    }
}

impl Eq for Stats {}

impl PartialOrd for Stats {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut part_ord = None;

        for ord in self
            .stats
            .iter()
            .chain(iter::repeat(&0))
            .zip(other.stats.iter().chain(iter::repeat(&0)))
            .map(|(s0, s1)| s0.cmp(s1))
            .take(self.stats.len().max(other.stats.len()))
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

    fn mul(mut self, rhs: u16) -> Self::Output {
        for stat in self.stats.iter_mut() {
            *stat *= rhs;
        }

        self
    }
}
