#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![deny(deprecated)]

pub mod dfs;
pub mod graph;
pub mod scroll;
pub mod stats;

#[test]
fn toy_of_101_test() {
    use crate::{
        dfs::solve_p, graph::ItemState, scroll::Scroll, stats::Stats,
    };

    let mut init_state =
        ItemState::new_exists(7, Stats::from_vec(vec![96, 3, 3, 0]));
    let scrolls = [
        Scroll::new(0.1, false, 100_000.0, Stats::from_vec(vec![5, 3, 0, 1])),
        Scroll::new(0.3, true, 1_300_000.0, Stats::from_vec(vec![5, 3, 0, 1])),
        Scroll::new(0.6, false, 40_000.0, Stats::from_vec(vec![2, 1, 0, 0])),
        Scroll::new(0.7, true, 45_000.0, Stats::from_vec(vec![2, 1, 0, 0])),
        Scroll::new(1.0, false, 70_000.0, Stats::from_vec(vec![1, 0, 0, 0])),
    ];

    solve_p(
        &mut init_state,
        &scrolls,
        &Stats::from_vec(vec![111, 0, 0, 0]),
    );

    if let ItemState::Exists {
        slots: _,
        stats: _,
        child,
    } = init_state
    {
        let child = child.unwrap();

        assert_eq!(
            child.scroll(),
            &Scroll::new(
                0.3,
                true,
                1_300_000.0,
                Stats::from_vec(vec![5, 3, 0, 1])
            )
        );
        assert_eq!(child.p_goal, 0.186_686_606_25);
        assert_eq!(child.exp_cost, 2_871_256.933_75);
    }
}
