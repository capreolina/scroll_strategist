use std::{cmp::Ordering, iter, ops::Mul};

/// The state of an item, including how many slots it has left, and what its
/// stats are. This is a node of a scrolling strategy tree, so it also can have
/// (or may not have) a single "child" of type `ScrollUse`. This only supports
/// one child at most, because we only want to keep the _optimal_ scroll usage
/// in memory; others should be, and are, discarded.
///
/// This is an enum because we want a way to represent the "state" of an item
/// that no longer exists, because it was boomed by a dark scroll.
enum ItemState<'a> {
    Exists {
        slots: u8,
        stats: Stats,
        child: Option<ScrollUse<'a>>,
    },
    Boomed,
}

impl<'a> ItemState<'a> {
    /// Creates a new instance of this type, specifically of the
    /// `ItemState::Exists` variant. The child is defaulted to `None`.
    pub const fn new_exists(slots: u8, stats: Stats) -> Self {
        Self::Exists {
            slots,
            stats,
            child: None,
        }
    }

    /// Creates a new instance of this type, specifically of the
    /// `ItemState::Boomed` variant. This contains no useful information other
    /// than that the item was boomed.
    pub const fn new_boomed() -> Self {
        Self::Boomed
    }
}

/// An instance of a particular scroll being used on a particular `ItemState`.
/// Like `ItemState`, this type represents a part of a scrolling strategy tree;
/// in particular, this represents a different kind of node than an `ItemState`
/// does.
///
/// The child nodes here are themselves `ItemStates`, representing all possible
/// outcomes of this scroll usage. The outcomes are stored in their own
/// special-sauce type, `Outcomes`.
///
/// This struct contains a member representing the probability of reaching the
/// goal given that this scroll is chosen (but not assuming any particular
/// outcome of the scroll). There is also a member of this struct representing
/// the expected cost (due solely to scroll expenditure) incurred due to this
/// scroll being used, in addition to all future scrolls used. The future
/// scroll costs are calculated optimally, as usual.
struct ScrollUse<'a> {
    /// "Probability of goal": Represents the probability of reaching the goal
    /// given that this scroll is chosen (but not assuming any particular
    /// outcome of the scroll).
    p_goal: f64,
    /// "Expected cost": Represents the expected cost (due solely to scroll
    /// expenditure) incurred due to this scroll being used, in addition to all
    /// future scrolls used. The future scroll costs are calculated optimally,
    /// as usual.
    exp_cost: f64,
    /// The scroll being used.
    scroll: &'a Scroll,
    /// All possible outcomes of this scroll usage; the children of this node.
    outcomes: Outcomes<'a>,
}

impl<'a> ScrollUse<'a> {
    /// Creates a new scroll usage struct, given a particular scroll that is
    /// being used. The probability of reaching the goal defaults to zero, the
    /// expected cost defaults to the cost of `scroll`, and there are no
    /// outcomes/children.
    pub const fn new(scroll: &'a Scroll) -> Self {
        Self {
            p_goal: 0.0,
            exp_cost: scroll.cost,
            scroll,
            outcomes: Outcomes::new(),
        }
    }
}

/// All possible outcomes of a particular scroll usage. Each outcome is
/// represented as an `ItemState`. See the documentation for `ScrollUse` (and
/// for `ItemState`) for more info.
#[derive(Default)]
struct Outcomes<'a> {
    outcomes: Vec<ItemState<'a>>,
}

impl<'a> Outcomes<'a> {
    /// Creates a new empty set of outcomes.
    pub const fn new() -> Self {
        Self {
            outcomes: Vec::new(),
        }
    }

    /// Adds a new outcome to this set of outcomes, and returns a mutable
    /// reference to the newly added outcome.
    pub fn push_outcome(
        &mut self,
        outcome: ItemState<'a>,
    ) -> &mut ItemState<'a> {
        self.outcomes.push(outcome);

        self.outcomes.last_mut().unwrap_or_else(|| unreachable!())
    }
}

/// A scroll. Contains all of the usual information associated with a scroll,
/// in addition to its nominal cost.
#[derive(Clone, Debug)]
struct Scroll {
    /// Probability of success.
    p_suc: f64,
    /// Is this a dark scroll?
    dark: bool,
    /// How much the scroll costs.
    cost: f64,
    /// What stats the scroll grants on success.
    stats: Stats,
}

impl Scroll {
    /// Creates a new scroll from the probability of success (`p_suc`), whether
    /// or not the scroll is dark (`dark`), what the nominal cost of the scroll
    /// is (`cost`), and what stats the scroll grants on success (`stats`).
    pub fn new(p_suc: f64, dark: bool, cost: f64, stats: Stats) -> Self {
        Self {
            p_suc,
            dark,
            cost,
            stats,
        }
    }

    /// Generates a "master scroll" based on a set of `Scroll`s. The master
    /// scroll has a 100% probability of success, is not a dark scroll, has a
    /// cost equal to [positive
    /// infinity](https://en.wikipedia.org/wiki/Extended_real_number_line), and
    /// grants a bonus to each stat that is equal to the highest bonus granted
    /// to that stat by any of the scrolls in the input set.
    ///
    /// Basically, the `stats` member of the master scroll is generated by
    /// summing up all of the `stats` members of the elements of the input set
    /// (`scrolls`), if you think of each `Stats` struct as an element of a
    /// [max tropical](https://en.wikipedia.org/wiki/Tropical_semiring)
    /// [semimodule](https://en.wikipedia.org/wiki/Semimodule).
    pub fn master_scroll(scrolls: &[Self]) -> Self {
        let mut master =
            Self::new(1.0, false, f64::INFINITY, Default::default());

        for scroll in scrolls {
            master.stats.max_in_place(&scroll.stats);
        }

        master
    }
}

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
struct Stats {
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
        let (longer, shorter) = if self.stats.len() >= other.stats.len() {
            (&self.stats, &other.stats)
        } else {
            (&other.stats, &self.stats)
        };

        let mut partial_ordering = None;

        for (i, stat) in longer.iter().enumerate() {
            let stat_cmp = stat.cmp(shorter.get(i).unwrap_or(&0));

            match partial_ordering {
                None => partial_ordering = Some(stat_cmp),
                Some(Ordering::Less) => {
                    if stat_cmp == Ordering::Greater {
                        return None;
                    }
                }
                Some(Ordering::Equal) => {
                    if stat_cmp != Ordering::Equal {
                        partial_ordering = Some(stat_cmp);
                    }
                }
                Some(Ordering::Greater) => {
                    if stat_cmp == Ordering::Less {
                        return None;
                    }
                }
            }
        }

        if self.stats.len() >= other.stats.len() {
            partial_ordering
        } else {
            partial_ordering.map(|o| match o {
                Ordering::Less => Ordering::Greater,
                Ordering::Greater => Ordering::Less,
                _ => o,
            })
        }
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

fn main() {
    let mut init_state =
        ItemState::new_exists(7, Stats::new_from_vec(vec![94]));
    let scrolls = [
        Scroll::new(0.1, false, 15_000.0, Stats::new_from_vec(vec![5, 3, 1])),
        Scroll::new(
            0.3,
            true,
            1_500_000.0,
            Stats::new_from_vec(vec![5, 3, 1]),
        ),
        Scroll::new(0.6, false, 50_000.0, Stats::new_from_vec(vec![2, 1])),
        Scroll::new(0.7, true, 30_000.0, Stats::new_from_vec(vec![2, 1])),
        Scroll::new(1.0, false, 70_000.0, Stats::new_from_vec(vec![1])),
    ];

    solve_p(&mut init_state, &scrolls, &Stats::new_from_vec(vec![110]));

    if let ItemState::Exists {
        slots: _,
        stats: _,
        child,
    } = init_state
    {
        let child = child.unwrap();

        println!(
            "scroll: {:?}\n\np_goal: {:?}\nexp_cost: {:?}",
            child.scroll, child.p_goal, child.exp_cost,
        );
    }
}

/// Like other search functions in this program, this function assumes that
/// `state` already has a well-defined value for `state.slots` and
/// `state.stats`. Also, if `state.child.is_some()`, the value inside of
/// `state.child` _will_ be ignored, and trampled/replaced. `scrolls` must be
/// nonempty.
///
/// This function optimises _only_ to maximise the probability of reaching
/// `goal`, going with lower expected costs only when needed to break a tie.
fn solve_p<'a>(
    state: &mut ItemState<'a>,
    scrolls: &'a [Scroll],
    goal: &Stats,
) {
    let master_scroll = Scroll::master_scroll(scrolls);
    dfs_p(state, scrolls, &master_scroll, goal);
}

/// Like other search functions in this program, this function assumes that
/// `state` already has a well-defined value for `state.slots` and
/// `state.stats`. Also, if `state.child.is_some()`, the value inside of
/// `state.child` _will_ be ignored, and trampled/replaced. `scrolls` must be
/// nonempty.
///
/// This version of DFS optimises _only_ to maximise the probability of
/// reaching `goal`, going with lower expected costs only when needed to break
/// a tie.
///
/// The `master_scroll` parameter is used solely for optimisation, i.e. it's
/// not _strictly_ necessary for this function to behave correctly.
///
/// ## Returns:
///
/// - Probability of reaching `goal`, assuming optimal scroll choices after
///   this point.
/// - Expected cost after this point, again assuming optimal scroll choices
///   after this point.
fn dfs_p<'a>(
    state: &mut ItemState<'a>,
    scrolls: &'a [Scroll],
    master_scroll: &Scroll,
    goal: &Stats,
) -> (f64, f64) {
    debug_assert!(!scrolls.is_empty());

    match state {
        ItemState::Exists {
            slots,
            stats,
            child,
        } => {
            let _ = child.take();

            if slots == &0 {
                return (if &*stats >= goal { 1.0 } else { 0.0 }, 0.0);
            }

            let slots_m1 = *slots - 1;

            for scroll in scrolls {
                let mut scroll_use = ScrollUse::new(scroll);

                if scroll.p_suc > 0.0 {
                    let outcome_suc_stats = stats.plus(&scroll.stats);

                    // Is it even possible to reach the goal at this point?
                    if &(outcome_suc_stats.plus(
                        &(master_scroll.stats.clone() * u16::from(slots_m1)),
                    )) < goal
                    {
                        continue;
                    }

                    let outcome_suc = scroll_use.outcomes.push_outcome(
                        ItemState::new_exists(slots_m1, outcome_suc_stats),
                    );

                    let (p_goal_cond_suc, exp_cost_cond_suc) =
                        dfs_p(outcome_suc, scrolls, master_scroll, goal);
                    scroll_use.p_goal += scroll.p_suc * p_goal_cond_suc;
                    scroll_use.exp_cost += scroll.p_suc * exp_cost_cond_suc;
                }

                // Is it even possible to reach the goal at this point?
                if &(stats.plus(
                    &(master_scroll.stats.clone() * u16::from(slots_m1)),
                )) < goal
                {
                    continue;
                }

                if scroll.p_suc < 1.0 {
                    let outcome_fail = scroll_use.outcomes.push_outcome(
                        ItemState::new_exists(slots_m1, stats.clone()),
                    );

                    let (p_goal_cond_fail, exp_cost_cond_fail) =
                        dfs_p(outcome_fail, scrolls, master_scroll, goal);
                    let p_fail = if scroll.dark {
                        (1.0 - scroll.p_suc) / 2.0
                    } else {
                        1.0 - scroll.p_suc
                    };
                    scroll_use.p_goal += p_fail * p_goal_cond_fail;
                    scroll_use.exp_cost += p_fail * exp_cost_cond_fail;

                    if scroll.dark {
                        let outcome_boom = scroll_use
                            .outcomes
                            .push_outcome(ItemState::new_boomed());

                        // These results are always `(0.0, 0.0)`, so we ignore
                        // them.
                        let (_p_goal_cond_boom, _exp_cost_cond_boom) =
                            dfs_p(outcome_boom, scrolls, master_scroll, goal);
                    }
                }

                if let Some(child_scroll_use) = child {
                    // We use the expected cost to break ties here.
                    if scroll_use.p_goal > child_scroll_use.p_goal
                        || (scroll_use.p_goal == child_scroll_use.p_goal
                            && scroll_use.exp_cost < child_scroll_use.exp_cost)
                    {
                        child.replace(scroll_use);
                    }
                } else {
                    child.replace(scroll_use);
                }
            }

            if let Some(child_scroll_use) = child.as_ref() {
                (child_scroll_use.p_goal, child_scroll_use.exp_cost)
            } else {
                (0.0, 0.0)
            }
        }
        ItemState::Boomed => (0.0, 0.0),
    }
}
