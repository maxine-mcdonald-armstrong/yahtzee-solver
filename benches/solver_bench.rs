use criterion::{Criterion, black_box, criterion_group, criterion_main};
use yahtzee_solver::types::{JokerRule, ScorecardState};
use yahtzee_solver::{MockScorecardMemo, dice_dp};

fn bench_dice_dp(c: &mut Criterion) {
    let scorecard_state = ScorecardState::default();
    let scorecard_memo = MockScorecardMemo();
    let joker_rule = JokerRule::FreeChoice;

    c.bench_function("dice_dp_full_pass", |b| {
        b.iter(|| black_box(dice_dp(&scorecard_state, &scorecard_memo, joker_rule)))
    });
}

criterion_group!(benches, bench_dice_dp);
criterion_main!(benches);
