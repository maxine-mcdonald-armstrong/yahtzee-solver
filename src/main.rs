use yahtzee_solver::types::{JokerRule, ScorecardState};
use yahtzee_solver::{MockScorecardMemo, dice_dp};

fn profile_dice_dp(n_runs: u32) {
    for _ in 0..n_runs {
        let scorecard_state = ScorecardState::default();
        let scorecard_memo = MockScorecardMemo();
        let joker_rule = JokerRule::FreeChoice;
        dice_dp(&scorecard_state, &scorecard_memo, joker_rule);
    }
}

fn main() {
    profile_dice_dp(100);
}
