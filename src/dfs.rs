use crate::{
    graph::{ItemState, ScrollUse},
    scroll::Scroll,
    stats::Stats,
};
use rustc_hash::FxHashMap;
use std::{
    hash::{Hash, Hasher},
    rc::Rc,
};

/// Like other search functions in this program, this function assumes that
/// `state` already has a well-defined value for `state.slots` and
/// `state.stats`. Also, if `state.child.is_some()`, the value inside of
/// `state.child` _will_ be ignored, and trampled/replaced. `scrolls` must be
/// nonempty.
///
/// This function optimises _only_ to maximise the probability of reaching
/// `goal`, going with lower expected costs only when needed to break a tie.
pub fn solve_p<'a>(
    state: &mut ItemState<'a>,
    scrolls: &'a [Scroll],
    goal: &Stats,
) {
    let master_scroll = Scroll::master_scroll(scrolls);
    let mut cache = Default::default();

    dfs_p(state, scrolls, &master_scroll, goal, &mut cache);
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
/// not _strictly_ necessary for this function to behave correctly. Likewise,
/// the `cache` parameter is also used solely for [dynamic
/// programming](https://en.wikipedia.org/wiki/Dynamic_programming)
/// optimisation.
///
/// ## Returns:
///
/// - Probability of reaching `goal`, assuming optimal scroll choices after
///   this point.
/// - Expected cost after this point, again assuming optimal scroll choices
///   after this point.
fn dfs_p<'a, 'b>(
    state: &'b mut ItemState<'a>,
    scrolls: &'a [Scroll],
    master_scroll: &Scroll,
    goal: &Stats,
    cache: &mut FxHashMap<CacheKey, Rc<ScrollUse<'a>>>,
) -> (f64, f64) {
    debug_assert!(!scrolls.is_empty());

    match state {
        ItemState::Exists {
            slots,
            stats,
            child,
        } => {
            if let Some(su) = cache.get(&CacheKey::new_borrowed(*slots, stats))
            {
                child.replace(Rc::clone(su));

                return (su.p_goal, su.exp_cost);
            }

            // Just in case `child.is_some()`.
            let _ = child.take();

            if slots == &0 {
                return (if &*stats >= goal { 1.0 } else { 0.0 }, 0.0);
            }

            let slots_m1 = *slots - 1;

            for scroll in scrolls {
                let mut scroll_use = ScrollUse::new(scroll);

                if scroll.p_suc > 0.0 {
                    // New stats of the item, assuming a success of this
                    // scroll.
                    let outcome_suc_stats = stats.plus(&scroll.stats);

                    // Is it even possible to reach the goal at this point?
                    // This is the "master scroll" heuristic.
                    if &(outcome_suc_stats.plus(
                        &(master_scroll.stats.clone() * u16::from(slots_m1)),
                    )) < goal
                    {
                        continue;
                    }

                    let outcome_suc = scroll_use.push_outcome(
                        ItemState::new_exists(slots_m1, outcome_suc_stats),
                    );

                    // (probability of reaching the goal conditioned on this
                    // scroll succeeding, expected cost after this point
                    // conditioned on this scroll succeeding)
                    let (p_goal_cond_suc, exp_cost_cond_suc) = dfs_p(
                        outcome_suc,
                        scrolls,
                        master_scroll,
                        goal,
                        cache,
                    );
                    scroll_use.p_goal += scroll.p_suc * p_goal_cond_suc;
                    scroll_use.exp_cost += scroll.p_suc * exp_cost_cond_suc;
                }

                // Is it even possible to reach the goal at this point? This is
                // the "master scroll" heuristic.
                if &(stats.plus(
                    &(master_scroll.stats.clone() * u16::from(slots_m1)),
                )) < goal
                {
                    continue;
                }

                if scroll.p_suc < 1.0 {
                    let outcome_fail = scroll_use.push_outcome(
                        ItemState::new_exists(slots_m1, stats.clone()),
                    );

                    // (probability of reaching the goal conditioned on this
                    // scroll failing, expected cost after this point
                    // conditioned on this scroll failing)
                    let (p_goal_cond_fail, exp_cost_cond_fail) = dfs_p(
                        outcome_fail,
                        scrolls,
                        master_scroll,
                        goal,
                        cache,
                    );
                    let p_fail = if scroll.dark {
                        (1.0 - scroll.p_suc) / 2.0
                    } else {
                        1.0 - scroll.p_suc
                    };
                    scroll_use.p_goal += p_fail * p_goal_cond_fail;
                    scroll_use.exp_cost += p_fail * exp_cost_cond_fail;

                    if scroll.dark {
                        let outcome_boom =
                            scroll_use.push_outcome(ItemState::new_boomed());

                        // These results are always `(0.0, 0.0)`, so we ignore
                        // them.
                        let (_p_goal_cond_boom, _exp_cost_cond_boom) = dfs_p(
                            outcome_boom,
                            scrolls,
                            master_scroll,
                            goal,
                            cache,
                        );
                    }
                }

                // Now, we check whether or not using this scroll is a better
                // choice than using any of the scrolls that we tested
                // previously.
                if let Some(child_scroll_use) = child {
                    // We use the expected cost to break ties here.
                    if scroll_use.p_goal > child_scroll_use.p_goal
                        || (scroll_use.p_goal >= child_scroll_use.p_goal
                            && scroll_use.exp_cost < child_scroll_use.exp_cost)
                    {
                        child.replace(Rc::new(scroll_use));
                    }
                } else {
                    child.replace(Rc::new(scroll_use));
                }
            }

            if let Some(child_scroll_use) = child.as_ref() {
                cache.insert(
                    CacheKey::new_owned(*slots, stats.clone()),
                    Rc::clone(child_scroll_use),
                );

                (child_scroll_use.p_goal, child_scroll_use.exp_cost)
            } else {
                // This branch is possibly taken when our "master scroll"
                // heuristic rejects all scrolls.
                (0.0, 0.0)
            }
        }
        ItemState::Boomed => (0.0, 0.0),
    }
}

/// This type exists specifically to avoid calling `Vec::clone` every time that
/// we do a lookup in the cache.
enum StatsHandle<'sh> {
    Owned(Stats),
    Borrowed(&'sh Stats),
}

impl<'sh> PartialEq for StatsHandle<'sh> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Owned(s0), Self::Owned(s1)) => s0 == s1,
            (Self::Owned(s0), Self::Borrowed(s1)) => &s0 == s1,
            (Self::Borrowed(s0), Self::Owned(s1)) => s0 == &s1,
            (Self::Borrowed(s0), Self::Borrowed(s1)) => s0 == s1,
        }
    }
}

impl<'sh> Eq for StatsHandle<'sh> {}

impl<'sh> Hash for StatsHandle<'sh> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Owned(s) => s.hash(state),
            Self::Borrowed(s) => s.hash(state),
        }
    }
}

/// This type exists specifically to avoid calling `Vec::clone` every time that
/// we do a lookup in the cache.
struct CacheKey<'sh> {
    slots: u8,
    stats: StatsHandle<'sh>,
}

impl<'sh> CacheKey<'sh> {
    fn new_owned(slots: u8, stats: Stats) -> Self {
        Self {
            slots,
            stats: StatsHandle::Owned(stats),
        }
    }

    fn new_borrowed(slots: u8, stats: &'sh Stats) -> Self {
        Self {
            slots,
            stats: StatsHandle::Borrowed(stats),
        }
    }
}

impl<'sh> PartialEq for CacheKey<'sh> {
    fn eq(&self, other: &Self) -> bool {
        self.slots == other.slots && self.stats == other.stats
    }
}

impl<'sh> Eq for CacheKey<'sh> {}

impl<'sh> Hash for CacheKey<'sh> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.slots.hash(state);
        self.stats.hash(state);
    }
}
