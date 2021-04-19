#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(deprecated)]

mod dfs;
mod graph;
mod scroll;
mod stats;

use crate::{dfs::solve_p, graph::ItemState, scroll::Scroll, stats::Stats};

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
            child.scroll(),
            child.p_goal,
            child.exp_cost,
        );
    }
}
