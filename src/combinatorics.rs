use crate::types::{KeepCounts, RollCounts, SubtractionError};

pub const DISTINCT_ROLL_COUNTS: usize =
    BINOM[RollCounts::NUM_DICE + RollCounts::NUM_FACES - 1][RollCounts::NUM_FACES - 1];
pub const DISTINCT_ROLLS: [[u8; RollCounts::NUM_FACES]; DISTINCT_ROLL_COUNTS] =
    make_distinct_rolls();
pub const DISTINCT_KEEP_COUNTS: usize = compute_distinct_keep_counts();
pub const DISTINCT_KEEPS: [[u8; RollCounts::NUM_FACES]; DISTINCT_KEEP_COUNTS] =
    make_distinct_keeps();

const MAX: usize = RollCounts::NUM_DICE + RollCounts::NUM_FACES;

/// Precomputed Pascal's triangle with 11 rows.
const BINOM: [[usize; MAX]; MAX] = make_binom();
impl RollCounts {
    /// Given a dice roll as a multiset, computes the rank of the multiset. A bijection between
    /// RollCounts <-> [0..C(10)(5)].
    ///
    /// * `roll_counts` - The dice roll multiset.
    pub fn rank(&self) -> usize {
        let mut rank: usize = 0;
        let mut dice_remaining: usize = RollCounts::NUM_DICE as usize;
        for face in 0..(RollCounts::NUM_DICE as usize) {
            let count = self.roll_counts()[face] as usize;
            for i in 0..count {
                let dice_left = dice_remaining - i;
                let faces_left = RollCounts::NUM_FACES as usize - face - 1;
                rank += BINOM[dice_left + faces_left - 1][faces_left - 1];
            }
            dice_remaining -= count;
        }
        rank
    }

    pub fn valid_keep_counts(&self) -> Vec<KeepCounts> {
        let mut valid_keep_counts = Vec::new();
        for k0 in 0..=self.roll_counts()[0] {
            for k1 in 0..=self.roll_counts()[1] {
                for k2 in 0..=self.roll_counts()[2] {
                    for k3 in 0..=self.roll_counts()[3] {
                        for k4 in 0..=self.roll_counts()[4] {
                            for k5 in 0..=self.roll_counts()[5] {
                                valid_keep_counts
                                    .push(KeepCounts::try_from([k0, k1, k2, k3, k4, k5]).expect(
                                            "Must be valid because it's a submultiset of a valid roll_counts.",
                                    ));
                            }
                        }
                    }
                }
            }
        }
        valid_keep_counts
    }

    pub fn p_roll(&self) -> f64 {
        self.p_roll_given_keep(&KeepCounts::try_from([0, 0, 0, 0, 0, 0]).unwrap())
    }

    pub fn p_roll_given_keep(&self, keep_counts: &KeepCounts) -> f64 {
        let Ok(to_reroll) = self.subtract(keep_counts) else {
            return 0f64;
        };
        let n_to_reroll = to_reroll.keep_counts().iter().sum::<u8>() as usize;
        let mut denominator: usize = 1;
        for &count in to_reroll.keep_counts().iter() {
            denominator *= factorial(count as usize);
        }
        denominator *= (RollCounts::NUM_FACES as usize).pow(n_to_reroll as u32);
        let numerator = factorial(n_to_reroll);
        let p = numerator as f64 / denominator as f64;
        p
    }
}

const fn factorial(n: usize) -> usize {
    let mut ans = 1;
    let mut i = 2;
    while i <= n {
        ans *= i;
        i += 1;
    }
    ans
}

const fn make_binom() -> [[usize; MAX]; MAX] {
    let mut binom = [[0; MAX]; MAX];
    let mut i = 0;
    while i < MAX {
        binom[i][0] = 1;
        binom[i][i] = 1;
        let mut j = 1;
        while j < i {
            binom[i][j] = binom[i - 1][j - 1] + binom[i - 1][j];
            j += 1;
        }
        i += 1;
    }
    binom
}

const fn compute_distinct_keep_counts() -> usize {
    let mut counts = 0usize;
    let mut n_kept = 0usize;
    while n_kept <= RollCounts::NUM_DICE {
        counts += BINOM[n_kept + RollCounts::NUM_FACES - 1][n_kept];
        n_kept += 1;
    }
    counts
}

/// We cannot return checked RollCount objects as that's not const.
const fn make_distinct_rolls() -> [[u8; RollCounts::NUM_FACES]; DISTINCT_ROLL_COUNTS] {
    let mut distinct_rolls = [[RollCounts::NUM_DICE as u8, 0, 0, 0, 0, 0]; DISTINCT_ROLL_COUNTS];
    let mut i = 0;
    let mut c0 = 0u8;
    while c0 <= RollCounts::NUM_DICE as u8 {
        let mut c1 = 0u8;
        while c0 + c1 <= RollCounts::NUM_DICE as u8 {
            let mut c2 = 0u8;
            while c0 + c1 + c2 <= RollCounts::NUM_DICE as u8 {
                let mut c3 = 0u8;
                while c0 + c1 + c2 + c3 <= RollCounts::NUM_DICE as u8 {
                    let mut c4 = 0u8;
                    while c0 + c1 + c2 + c3 + c4 <= RollCounts::NUM_DICE as u8 {
                        let c5 = RollCounts::NUM_DICE as u8 - c0 - c1 - c2 - c3 - c4;
                        distinct_rolls[i] = [c0, c1, c2, c3, c4, c5];
                        i += 1;
                        c4 += 1;
                    }
                    c3 += 1;
                }
                c2 += 1;
            }
            c1 += 1;
        }
        c0 += 1;
    }
    distinct_rolls
}

const fn make_distinct_keeps() -> [[u8; RollCounts::NUM_FACES]; DISTINCT_KEEP_COUNTS] {
    let mut distinct_keeps = [[0, 0, 0, 0, 0, 0]; DISTINCT_KEEP_COUNTS];
    let mut i = 0;
    let mut c0 = 0u8;
    while c0 <= RollCounts::NUM_DICE as u8 {
        let mut c1 = 0u8;
        while c0 + c1 <= RollCounts::NUM_DICE as u8 {
            let mut c2 = 0u8;
            while c0 + c1 + c2 <= RollCounts::NUM_DICE as u8 {
                let mut c3 = 0u8;
                while c0 + c1 + c2 + c3 <= RollCounts::NUM_DICE as u8 {
                    let mut c4 = 0u8;
                    while c0 + c1 + c2 + c3 + c4 <= RollCounts::NUM_DICE as u8 {
                        let mut c5 = 0u8;
                        while c0 + c1 + c2 + c3 + c4 + c5 <= RollCounts::NUM_DICE as u8 {
                            distinct_keeps[i] = [c0, c1, c2, c3, c4, c5];
                            c5 += 1;
                            i += 1;
                        }
                        c4 += 1;
                    }
                    c3 += 1;
                }
                c2 += 1;
            }
            c1 += 1;
        }
        c0 += 1;
    }
    distinct_keeps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_is_bijection() {
        let mut rank_to_roll_counts: [Option<RollCounts>; DISTINCT_ROLL_COUNTS] =
            [None; DISTINCT_ROLL_COUNTS];
        for distinct_roll in DISTINCT_ROLLS {
            let roll_counts = RollCounts::try_from(distinct_roll)
                .expect("These were created in such a way that they should all be valid.");
            let rank = roll_counts.rank();
            if rank_to_roll_counts[rank] == None {
                rank_to_roll_counts[rank] = Some(roll_counts);
                continue;
            }
            if rank_to_roll_counts[rank] == Some(roll_counts) {
                continue;
            }
            assert!(
                false,
                "rank not injective: {:#?} collided with {:#?} at rank {}.",
                roll_counts, rank_to_roll_counts[rank], rank
            );
        }
        // Loop has ended => No collisions (injective). Now we check that every rank is assigned a roll count (surjective).
        assert!(
            !rank_to_roll_counts.iter().any(|&x| x == None),
            "rank not surjective: {:#?}",
            rank_to_roll_counts
        );
    }

    #[test]
    fn test_p_roll() {
        let cases = [
            ([5, 0, 0, 0, 0, 0], 1.0 / 6f64.powf(5.0)),
            ([0, 0, 0, 5, 0, 0], 1.0 / 6f64.powf(5.0)),
            ([1, 1, 1, 1, 1, 0], 5.0 * 4.0 * 3.0 * 2.0 / 6f64.powf(5.0)),
            ([1, 1, 0, 1, 1, 1], 5.0 * 4.0 * 3.0 * 2.0 / 6f64.powf(5.0)),
        ];
        for (roll, expected) in cases {
            let roll_counts = RollCounts::try_from(roll).unwrap();
            let p = roll_counts.p_roll();
            assert!(
                (p - expected).abs() < f64::EPSILON,
                "Calculated p={}, expected p={}, difference larger than epsilon {} at {}.",
                p,
                expected,
                f64::EPSILON,
                (p - expected).abs()
            );
        }
    }

    #[test]
    fn test_p_roll_given_keep() {
        let cases = [
            ([5, 0, 0, 0, 0, 0], [4, 0, 0, 0, 0, 0], 1.0 / 6f64),
            ([5, 0, 0, 0, 0, 0], [3, 0, 0, 0, 0, 0], 1.0 / 36f64),
            ([1, 0, 1, 1, 1, 1], [2, 0, 0, 0, 0, 0], 0f64),
        ];
        for (roll, keep, expected) in cases {
            let roll_counts = RollCounts::try_from(roll).unwrap();
            let keep_counts = KeepCounts::try_from(keep).unwrap();
            let p = roll_counts.p_roll_given_keep(&keep_counts);
            assert!(
                (p - expected).abs() < f64::EPSILON,
                "Calculated p={}, expected p={}, difference larger than epsilon {} at {}.",
                p,
                expected,
                f64::EPSILON,
                (p - expected).abs()
            );
        }
    }

    #[test]
    fn test_valid_keep_counts() {
        let roll_counts = RollCounts::try_from([5, 0, 0, 0, 0, 0]).unwrap();
        let expected = vec![
            KeepCounts::try_from([0, 0, 0, 0, 0, 0]).unwrap(),
            KeepCounts::try_from([1, 0, 0, 0, 0, 0]).unwrap(),
            KeepCounts::try_from([2, 0, 0, 0, 0, 0]).unwrap(),
            KeepCounts::try_from([3, 0, 0, 0, 0, 0]).unwrap(),
            KeepCounts::try_from([4, 0, 0, 0, 0, 0]).unwrap(),
            KeepCounts::try_from([5, 0, 0, 0, 0, 0]).unwrap(),
        ];
        assert_eq!(expected, roll_counts.valid_keep_counts());
    }
}
