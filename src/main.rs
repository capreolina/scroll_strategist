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
    fn new(scroll: &'a Scroll) -> Self {
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
    pub fn push_outcome(&mut self, outcome: ItemState<'a>) {
        self.outcomes.push(outcome);
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
}

impl Stats {
    pub fn plus(&self, other: &Self) -> Self {
        let mut stats =
            Vec::with_capacity(self.stats.len().max(other.stats.len()));

        if self.stats.len() >= other.stats.len() {
            for (i, stat) in self.stats.iter().enumerate() {
                stats.push(stat + other.stats.get(i).unwrap_or(&0));
            }
        } else {
            for (i, stat) in other.stats.iter().enumerate() {
                stats.push(stat + self.stats.get(i).unwrap_or(&0));
            }
        }

        Self { stats }
    }
}

fn main() {
    let mut init_state =
        ItemState::new_exists(7, Stats::new_from_vec(vec![100]));

    dfs(&mut init_state, &[], &Stats::new_from_vec(vec![110]));
}

fn dfs(state: &mut ItemState, scrolls: &[Scroll], goal: &Stats) {
    if let ItemState::Exists {
        slots,
        stats,
        children,
    } = state
    {
        debug_assert!(children.is_empty());

        if slots == &0 {
            return;
        }

        for scroll in scrolls {
            let mut scroll_use = ScrollUse::new(scroll);

            if scroll.p_suc > 0.0 {
                scroll_use.outcomes.push_outcome(ItemState::new_exists(
                    *slots - 1,
                    stats.plus(&scroll.stats),
                ));
            }

            if scroll.p_suc < 1.0 {
                scroll_use.outcomes.push_outcome(ItemState::new_exists(
                    *slots - 1,
                    stats.clone(),
                ));

                if scroll.dark {
                    scroll_use.outcomes.push_outcome(ItemState::new_boomed());
                }
            }

            for outcome in scroll_use.outcomes.iter_mut() {
                dfs(outcome, scrolls, goal);
            }
        }
    }
}
