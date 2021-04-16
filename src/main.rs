use std::cmp::Ordering;

enum ItemState<'a> {
    Exists {
        slots: u8,
        stats: Stats,
        child: Option<ScrollUse<'a>>,
    },
    Boomed,
}

impl<'a> ItemState<'a> {
    pub fn new_exists(slots: u8, stats: Stats) -> Self {
        Self::Exists {
            slots,
            stats,
            child: None,
        }
    }

    pub fn new_boomed() -> Self {
        Self::Boomed
    }
}

struct ScrollUse<'a> {
    p_goal: f64,
    exp_cost: f64,
    scroll: &'a Scroll,
    outcomes: Outcomes<'a>,
}

impl<'a> ScrollUse<'a> {
    pub fn new(scroll: &'a Scroll) -> Self {
        Self {
            p_goal: 0.0,
            exp_cost: scroll.cost,
            scroll,
            outcomes: Default::default(),
        }
    }
}

#[derive(Default)]
struct Outcomes<'a> {
    outcomes: Vec<ItemState<'a>>,
}

impl<'a> Outcomes<'a> {
    pub fn push_outcome(
        &mut self,
        outcome: ItemState<'a>,
    ) -> &mut ItemState<'a> {
        self.outcomes.push(outcome);

        self.outcomes.last_mut().unwrap_or_else(|| unreachable!())
    }
}

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
    pub fn new(p_suc: f64, dark: bool, cost: f64, stats: Stats) -> Self {
        Self {
            p_suc,
            dark,
            cost,
            stats,
        }
    }

    pub fn zero_percent() -> Self {
        Self {
            p_suc: 0.0,
            dark: false,
            cost: 0.0,
            stats: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct Stats {
    stats: Vec<u16>,
}

impl Stats {
    pub fn new_from_vec(stats: Vec<u16>) -> Self {
        Self { stats }
    }

    pub fn plus(&self, other: &Self) -> Self {
        let (longer, shorter) = if self.stats.len() >= other.stats.len() {
            (&self.stats, &other.stats)
        } else {
            (&other.stats, &self.stats)
        };

        let mut stats = Vec::with_capacity(longer.len());

        for (i, stat) in longer.iter().enumerate() {
            stats.push(stat + shorter.get(i).unwrap_or(&0));
        }

        Self { stats }
    }
}

impl PartialEq for Stats {
    fn eq(&self, other: &Self) -> bool {
        let (longer, shorter) = if self.stats.len() >= other.stats.len() {
            (&self.stats, &other.stats)
        } else {
            (&other.stats, &self.stats)
        };

        for (i, stat) in longer.iter().enumerate() {
            if stat != shorter.get(i).unwrap_or(&0) {
                return false;
            }
        }

        true
    }
}

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

fn main() {
    let mut init_state =
        ItemState::new_exists(7, Stats::new_from_vec(vec![94]));
    let scrolls = [
        Scroll::zero_percent(),
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

    let (p_goal, exp_cost) =
        dfs_p(&mut init_state, &scrolls, &Stats::new_from_vec(vec![110]));

    println!("p_goal, exp_cost: {}, {}\n", p_goal, exp_cost);

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
/// This version of DFS optimises _only_ to maximise the probability of
/// reaching `goal`, going with lower expected costs only when needed to break
/// a tie.
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
                    let outcome_suc = scroll_use.outcomes.push_outcome(
                        ItemState::new_exists(
                            slots_m1,
                            stats.plus(&scroll.stats),
                        ),
                    );

                    let (p_goal_cond_suc, exp_cost_cond_suc) =
                        dfs_p(outcome_suc, scrolls, goal);
                    scroll_use.p_goal += scroll.p_suc * p_goal_cond_suc;
                    scroll_use.exp_cost += scroll.p_suc * exp_cost_cond_suc;
                }

                if scroll.p_suc < 1.0 {
                    let outcome_fail = scroll_use.outcomes.push_outcome(
                        ItemState::new_exists(slots_m1, stats.clone()),
                    );

                    let (p_goal_cond_fail, exp_cost_cond_fail) =
                        dfs_p(outcome_fail, scrolls, goal);
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

                        let (p_goal_cond_boom, exp_cost_cond_boom) =
                            dfs_p(outcome_boom, scrolls, goal);
                        let p_boom = (1.0 - scroll.p_suc) / 2.0;
                        scroll_use.p_goal += p_boom * p_goal_cond_boom;
                        scroll_use.exp_cost += p_boom * exp_cost_cond_boom;
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

            let child_scroll_use = child.as_ref().unwrap();

            (child_scroll_use.p_goal, child_scroll_use.exp_cost)
        }
        ItemState::Boomed => (0.0, 0.0),
    }
}
