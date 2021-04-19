use crate::{scroll::Scroll, stats::Stats};
use std::rc::Rc;

/// The state of an item, including how many slots it has left, and what its
/// stats are. This is a node of a scrolling strategy tree, so it also can have
/// (or may not have) a single "child" of type `ScrollUse`. This only supports
/// one child at most, because we only want to keep the _optimal_ scroll usage
/// in memory; others should be, and are, discarded.
///
/// This is an enum because we want a way to represent the "state" of an item
/// that no longer exists, because it was boomed by a dark scroll.
pub enum ItemState<'a> {
    Exists {
        slots: u8,
        stats: Stats,
        child: Option<Rc<ScrollUse<'a>>>,
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
pub struct ScrollUse<'a> {
    /// "Probability of goal": Represents the probability of reaching the goal
    /// given that this scroll is chosen (but not assuming any particular
    /// outcome of the scroll).
    pub p_goal: f64,
    /// "Expected cost": Represents the expected cost (due solely to scroll
    /// expenditure) incurred due to this scroll being used, in addition to all
    /// future scrolls used. The future scroll costs are calculated optimally,
    /// as usual.
    pub exp_cost: f64,
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

    /// Returns a reference to the scroll being used here.
    pub const fn scroll(&self) -> &'a Scroll {
        self.scroll
    }

    /// Adds a new outcome to this `ScrollUse`'s set of outcomes, and returns a
    /// mutable reference to the newly added outcome.
    pub fn push_outcome(
        &mut self,
        outcome: ItemState<'a>,
    ) -> &mut ItemState<'a> {
        self.outcomes.push_outcome(outcome)
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
