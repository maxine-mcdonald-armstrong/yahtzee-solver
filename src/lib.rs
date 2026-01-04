pub mod combinatorics;
pub mod types;
pub mod yahtzee;

use combinatorics::{DISTINCT_KEEPS, DISTINCT_ROLL_COUNTS, DISTINCT_ROLLS};
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::LazyLock;
use strum::IntoEnumIterator;
use types::{
    DiceState, JokerRule, KeepCounts, RollCounts, RollsLeft, ScoreCategory, ScorecardState,
};

static ROLL_PROBABILITIES: LazyLock<VecMemo<KeepCounts, Vec<(usize, f64)>>> =
    LazyLock::new(|| precompute_roll_probabilities_vec());
static VALID_KEEP_COUNTS: LazyLock<VecMemo<RollCounts, Vec<KeepCounts>>> =
    LazyLock::new(|| precompute_valid_keep_counts_vec());

/// Allows me to easily swap out different memo implementations for the DP.
///
/// This might be a slightly contrived use case, but I wanted to learn about traits.
pub trait Memo<K, V> {
    /// Returns &memo[key] if it exists.
    fn get(&self, key: &K) -> Option<&V>;
    /// Sets memo[key] = value, returning whatever was previously there.
    fn set(&mut self, key: K, value: V) -> Option<V>;
    /// Removes memo[key], returning whatever was previously there.
    fn remove(&mut self, key: &K) -> Option<V>;
}

/// Allows any key to be used with a VecMemo as long as some to_index and max_index are
/// implemented.
pub trait IndexKey {
    fn to_index(&self) -> usize;
    fn max_index() -> usize;
}

pub struct MapMemo<K, V> {
    memo: HashMap<K, V>,
}

pub struct VecMemo<K, V> {
    memo: Vec<Option<V>>,
    _phantom: PhantomData<K>,
}

/// Used for profiling dice_dp
pub struct MockScorecardMemo();

impl<K: Eq + Hash, V> Memo<K, V> for MapMemo<K, V> {
    fn get(&self, key: &K) -> Option<&V> {
        self.memo.get(key)
    }

    fn set(&mut self, key: K, value: V) -> Option<V> {
        self.memo.insert(key, value)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.memo.remove(key)
    }
}

impl<K, V> Default for MapMemo<K, V> {
    fn default() -> Self {
        Self {
            memo: HashMap::new(),
        }
    }
}

impl<K: IndexKey, V> Memo<K, V> for VecMemo<K, V> {
    fn get(&self, key: &K) -> Option<&V> {
        self.memo[key.to_index()].as_ref()
    }

    fn set(&mut self, key: K, value: V) -> Option<V> {
        let prev = self.memo[key.to_index()].take();
        self.memo[key.to_index()] = Some(value);
        prev
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.memo[key.to_index()].take()
    }
}

impl<K: IndexKey, V> VecMemo<K, V> {
    fn new() -> Self {
        let size = K::max_index() + 1;
        let mut vec = Vec::with_capacity(size);
        vec.resize_with(size, || None);
        Self {
            memo: vec,
            _phantom: PhantomData,
        }
    }

    fn raw_get(&self, key: usize) -> Option<&V> {
        self.memo[key].as_ref()
    }

    fn raw_set(&mut self, key: usize, value: V) -> Option<V> {
        let prev = self.memo[key].take();
        self.memo[key] = Some(value);
        prev
    }

    fn raw_remove(&mut self, key: usize) -> Option<V> {
        self.memo[key].take()
    }
}

impl Memo<ScorecardState, f64> for MockScorecardMemo {
    fn get(&self, key: &ScorecardState) -> Option<&f64> {
        Some(&10f64)
    }

    fn set(&mut self, key: ScorecardState, value: f64) -> Option<f64> {
        Some(10f64)
    }

    fn remove(&mut self, key: &ScorecardState) -> Option<f64> {
        Some(10f64)
    }
}

impl IndexKey for DiceState {
    /// We use stars-and-bars to give each roll a unique number on [0..252], and then consider
    /// rolls_left which is on [0..3].
    fn to_index(&self) -> usize {
        let rank = self.roll_counts.rank();
        rank * (RollsLeft::MAX as usize + 1) + *self.rolls_left.rolls_left() as usize
    }

    fn max_index() -> usize {
        DISTINCT_ROLL_COUNTS * (RollsLeft::MAX as usize + 1) - 1
    }
}

impl IndexKey for RollCounts {
    fn to_index(&self) -> usize {
        self.rank()
    }

    fn max_index() -> usize {
        DISTINCT_ROLL_COUNTS - 1
    }
}

impl IndexKey for KeepCounts {
    /// These indices are very sparsely distributed, so this should be used with caution.
    fn to_index(&self) -> usize {
        let mut rank = 0usize;
        for &c in self.keep_counts() {
            rank *= 5;
            rank += c as usize;
        }
        rank
    }

    fn max_index() -> usize {
        5usize.pow(6)
    }
}

impl KeepCounts {
    /// For these dice kept, the rank of all possible rollcounts as a result of rerolling and their
    /// probabilities.
    fn roll_probabilities(&self) -> Vec<(usize, f64)> {
        let mut vec: Vec<(usize, f64)> = Vec::new();
        for raw_target_roll_counts in DISTINCT_ROLLS {
            let target_roll_counts = RollCounts::try_from(raw_target_roll_counts).unwrap();
            let p = target_roll_counts.p_roll_given_keep(&self);
            vec.push((target_roll_counts.rank(), p));
        }
        vec
    }
}

fn precompute_roll_probabilities_vec() -> VecMemo<KeepCounts, Vec<(usize, f64)>> {
    let mut memo: VecMemo<KeepCounts, Vec<(usize, f64)>> = VecMemo::new();
    for raw_keep_counts in DISTINCT_KEEPS {
        let keep_counts = KeepCounts::try_from(raw_keep_counts).unwrap();
        let roll_probabilities = keep_counts.roll_probabilities();
        memo.set(keep_counts, roll_probabilities);
    }
    memo
}

fn precompute_valid_keep_counts_vec() -> VecMemo<RollCounts, Vec<KeepCounts>> {
    let mut memo: VecMemo<RollCounts, Vec<KeepCounts>> = VecMemo::new();
    for raw_roll_counts in DISTINCT_ROLLS {
        let roll_counts = RollCounts::try_from(raw_roll_counts).unwrap();
        let valid_keep_counts = roll_counts.valid_keep_counts();
        memo.set(roll_counts, valid_keep_counts);
    }
    memo
}

/// Finds the EV of the given scorecard state. Does this by solving a finite-horizon MDP TC. Also
/// returns the optimal policy from any dice state given this scorecard state, because it's
/// annoying to recreate from the EV memo.
///
/// Requires the EV of all downstream scorecard states to be calculated. Is intended to be used as
/// a helper in scorecard_dp.
///
/// * `scorecard_state` - The state to solve the dice DP on.
/// * `initial_value` - A value of type V to initialize the optimisation on. This should be the
/// minimal possible V, for example 0.0 for Yahtzee (as negative scores are impossible).
/// * `scorecard_memo` - The current memo of ScorecardState -> V.
pub fn dice_dp<S: Memo<ScorecardState, f64>>(
    scorecard_state: &ScorecardState,
    scorecard_memo: &S,
    joker_rule: JokerRule,
) -> (
    impl Memo<DiceState, f64>,
    impl Memo<DiceState, &'static KeepCounts>,
) {
    let mut ev_memo: VecMemo<DiceState, f64> = VecMemo::new();
    let mut policy_memo: VecMemo<DiceState, &KeepCounts> = VecMemo::new();
    // initialise memo with all transitions out of this scorecard_state
    for raw_roll_counts in DISTINCT_ROLLS {
        let roll_counts = RollCounts::try_from(raw_roll_counts).expect("DISTINCT_ROLLS returns all valid roll counts, but due to its const implementation needs to be coerced at runtime.");
        for raw_rolls_left in 0..=RollsLeft::MAX {
            let rolls_left = RollsLeft::try_from(raw_rolls_left)
                .expect("We took care to iterate only over valid values.");
            let dice_state = DiceState {
                roll_counts: roll_counts,
                rolls_left: rolls_left,
            };
            // for each direct transition, the EV is the immediate score + the EV of that
            // transition.
            let mut best_ev = 0f64;
            for score_category in scorecard_state.valid_score_categories(&roll_counts, joker_rule) {
                let (category_score, bonus_score) = scorecard_state
                    .score_value(&roll_counts, score_category, joker_rule)
                    .expect("We are iterating through valid categories.");
                let target_scorecard_state = scorecard_state
                    .score(score_category, category_score)
                    .expect("This is a valid score category.");
                let transition_ev = if !target_scorecard_state.is_terminal() {
                    scorecard_memo
                        .get(&target_scorecard_state
                        )
                        .copied()
                        .expect("Our scorecard DP is working backwards, so every valid transition must be accounted for.")
                } else {
                    0f64
                };
                let total_ev = (category_score + bonus_score) as f64 + transition_ev;
                if total_ev > best_ev {
                    best_ev = total_ev;
                }
            }
            ev_memo.set(dice_state, best_ev);
            // for our policy memo, an element not being in it means that the optimal policy is
            // scoring that dice state.
        }
    }
    // Note that since every state is a potential terminal state (we can choose to score our dice
    // at any time), we must check whether immediate scoring has higher EV than the EV of any
    // expected transition.
    for raw_roll_counts in DISTINCT_ROLLS {
        let roll_counts = RollCounts::try_from(raw_roll_counts).unwrap();
        for raw_rolls_left in 1..=RollsLeft::MAX {
            let rolls_left = RollsLeft::try_from(raw_rolls_left).unwrap();
            let dice_state = DiceState {
                roll_counts: roll_counts,
                rolls_left: rolls_left,
            };
            // The following ensures we choose to score prematurely if it's optimal. It is what
            // necessitates the earlier loop that calculates the EV of scoring each possible dice
            // state.
            let mut best_ev = ev_memo
                .get(&dice_state)
                .copied()
                .expect("We initialized the memo with every possible dicestate");
            let mut best_transition: Option<&KeepCounts> = None;
            // over all possible dice transitions (keep_counts)...
            for keep_counts in VALID_KEEP_COUNTS.get(&roll_counts).unwrap() {
                // calculate the EV of following that transition
                let mut ev = 0f64;
                for (target_roll_counts_rank, p) in ROLL_PROBABILITIES.get(keep_counts).unwrap() {
                    let target_rolls_left = (raw_rolls_left - 1) as usize;
                    let memo_idx =
                        target_roll_counts_rank * (RollsLeft::MAX as usize + 1) + target_rolls_left;
                    ev = ev + p * ev_memo.raw_get(memo_idx).copied().expect("Our dice DP is working backwards, so every valid transition must be accounted for.");
                }
                if ev > best_ev {
                    best_ev = ev;
                    best_transition = Some(keep_counts);
                }
            }
            ev_memo.set(dice_state.clone(), best_ev);
            if let Some(concrete_transition) = best_transition {
                policy_memo.set(dice_state, concrete_transition);
            }
        }
    }
    (ev_memo, policy_memo)
}

/// Builds the scorecard DP memo from ScorecardState -> EV.
pub fn scorecard_dp() -> impl Memo<ScorecardState, f64> {
    let mut memo: MapMemo<ScorecardState, f64> = MapMemo::default();
    todo!();
    memo
}
