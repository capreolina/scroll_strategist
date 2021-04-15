use std::cmp::Ordering;

enum ItemState<'a> {
    Exists {
        slots: u8,
        stats: Stats,
        children: Vec<ScrollUse<'a>>,
    },
    Boomed,
}

impl<'a> ItemState<'a> {
    pub fn new_exists(slots: u8, stats: Stats) -> Self {
        Self::Exists {
            slots,
            stats,
            children: Vec::new(),
        }
    }

    pub fn new_boomed() -> Self {
        Self::Boomed
    }
}

struct ScrollUse<'a> {
    p_suc: Option<f64>,
    cost_exp: Option<f64>,
    scroll: &'a Scroll,
    outcomes: Outcomes<'a>,
}

impl<'a> ScrollUse<'a> {
    pub fn new(scroll: &'a Scroll) -> Self {
        Self {
            p_suc: None,
            cost_exp: None,
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

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ItemState<'a>> {
        self.outcomes.iter_mut()
    }
}

struct Scroll {
    p_suc: f64,
    dark: bool,
    cost: f64,
    stats: Stats,
}

#[derive(Clone, Default)]
struct Stats {
    stats: Vec<u16>,
}

impl Stats {
    pub fn new() -> Self {
        Self { stats: Vec::new() }
    }

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
        ItemState::new_exists(7, Stats::new_from_vec(vec![100]));

    dfs(&mut init_state, &[], &Stats::new_from_vec(vec![110]));
}

/// Returns:
///
/// - Probability of meeting `goal`
/// - Expected cost after this point
fn dfs<'a>(
    state: &mut ItemState<'a>,
    scrolls: &'a [Scroll],
    goal: &Stats,
) -> (f64, f64) {
    match state {
        ItemState::Exists {
            slots,
            stats,
            children,
        } => {
            debug_assert!(children.is_empty());

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

                    dfs(outcome_suc, scrolls, goal);
                }

                if scroll.p_suc < 1.0 {
                    scroll_use.outcomes.push_outcome(ItemState::new_exists(
                        slots_m1,
                        stats.clone(),
                    ));

                    if scroll.dark {
                        scroll_use
                            .outcomes
                            .push_outcome(ItemState::new_boomed());
                    }
                }

                children.push(scroll_use);
            }

            // ...
        }
        ItemState::Boomed => (0.0, 0.0),
    }
}
