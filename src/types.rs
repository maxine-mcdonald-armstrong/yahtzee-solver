use enum_map::{Enum, EnumMap, enum_map};
use strum_macros::EnumIter;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum JokerRule {
    Forced,
    FreeChoice,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DiceState {
    pub roll_counts: RollCounts,
    pub rolls_left: RollsLeft,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
pub struct ScorecardState {
    pub capped_upper_section_score: CappedUpperSectionScore,
    score_category_state: EnumMap<ScoreCategory, ScoreCategoryState>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RollCounts([u8; RollCounts::NUM_FACES]);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RollsLeft(u8);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct KeepCounts([u8; RollCounts::NUM_FACES]);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct CappedUpperSectionScore(u8);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum, EnumIter)]
pub enum ScoreCategory {
    Aces,
    Twos,
    Threes,
    Fours,
    Fives,
    Sixes,
    FullHouse,
    ThreeOfAKind,
    FourOfAKind,
    SmallStraight,
    LargeStraight,
    Yahtzee,
    Chance,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
pub enum ScoreCategoryState {
    #[default]
    Unscored,
    Scored,
    Scratched,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ConstructionError {
    ValueTooLarge { max: u8, got: u8 },
    SumMismatch { expected: u8, got: u8 },
}

#[derive(Debug, Eq, PartialEq)]
pub enum ScoringError {
    InvalidScoreCategory { category: ScoreCategory },
}

#[derive(Debug, Eq, PartialEq)]
pub enum SubtractionError {
    Underflow { index: u8 },
}

impl ScorecardState {
    pub fn score_category_state(&self) -> &EnumMap<ScoreCategory, ScoreCategoryState> {
        &self.score_category_state
    }

    pub fn score(&self, category: ScoreCategory, score: u8) -> Result<Self, ScoringError> {
        if self.score_category_state()[category] != ScoreCategoryState::Unscored {
            return Err(ScoringError::InvalidScoreCategory { category: category });
        }
        let mut new_score_category_state = *self.score_category_state();
        if category == ScoreCategory::Yahtzee && score == 0 {
            new_score_category_state[category] = ScoreCategoryState::Scratched;
        } else {
            new_score_category_state[category] = ScoreCategoryState::Scored;
        }
        let mut new_capped_upper_section_score = self.capped_upper_section_score;
        if category.is_upper_section() {
            new_capped_upper_section_score = new_capped_upper_section_score.add_clamped(score);
        }
        Ok(Self {
            capped_upper_section_score: new_capped_upper_section_score,
            score_category_state: new_score_category_state,
        })
    }
}

impl ScoreCategory {
    pub fn is_upper_section(&self) -> bool {
        matches!(
            self,
            ScoreCategory::Aces
                | ScoreCategory::Twos
                | ScoreCategory::Threes
                | ScoreCategory::Fours
                | ScoreCategory::Fives
                | ScoreCategory::Sixes
        )
    }

    pub fn is_lower_section(&self) -> bool {
        !self.is_upper_section()
    }
}

impl RollCounts {
    pub const NUM_DICE: usize = 5;
    pub const NUM_FACES: usize = 6;

    pub fn roll_counts(&self) -> &[u8; Self::NUM_FACES] {
        &self.0
    }

    pub fn subtract(&self, keep_counts: &KeepCounts) -> Result<KeepCounts, SubtractionError> {
        let mut result = *self.roll_counts();
        for i in 0..RollCounts::NUM_DICE {
            if result[i] < keep_counts.keep_counts()[i] {
                return Err(SubtractionError::Underflow { index: i as u8 });
            }
            result[i] -= keep_counts.keep_counts()[i];
        }
        Ok(KeepCounts::try_from(result).unwrap())
    }
}

impl TryFrom<[u8; RollCounts::NUM_FACES]> for RollCounts {
    type Error = ConstructionError;

    fn try_from(value: [u8; Self::NUM_FACES]) -> Result<Self, Self::Error> {
        if value.iter().any(|&x| x > Self::NUM_DICE as u8) {
            Err(Self::Error::ValueTooLarge {
                max: Self::NUM_DICE as u8,
                got: *value
                    .iter()
                    .filter(|&&x| x > Self::NUM_DICE as u8)
                    .next()
                    .unwrap(),
            })
        } else if value.iter().sum::<u8>() != Self::NUM_DICE as u8 {
            Err(Self::Error::SumMismatch {
                expected: Self::NUM_DICE as u8,
                got: value.iter().sum::<u8>(),
            })
        } else {
            Ok(Self(value))
        }
    }
}

impl RollsLeft {
    pub const MAX: u8 = 2;

    pub fn rolls_left(&self) -> &u8 {
        &self.0
    }
}

impl TryFrom<u8> for RollsLeft {
    type Error = ConstructionError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 2 {
            Err(Self::Error::ValueTooLarge { max: 2, got: value })
        } else {
            Ok(Self(value))
        }
    }
}

impl KeepCounts {
    pub fn keep_counts(&self) -> &[u8; RollCounts::NUM_FACES] {
        &self.0
    }
}

impl TryFrom<[u8; RollCounts::NUM_FACES]> for KeepCounts {
    type Error = ConstructionError;

    fn try_from(value: [u8; RollCounts::NUM_FACES]) -> Result<Self, Self::Error> {
        if value.iter().any(|&x| x > RollCounts::NUM_DICE as u8) {
            Err(Self::Error::ValueTooLarge {
                max: RollCounts::NUM_DICE as u8,
                got: *value
                    .iter()
                    .filter(|&&x| x > RollCounts::NUM_DICE as u8)
                    .next()
                    .unwrap(),
            })
        } else if value.iter().sum::<u8>() > RollCounts::NUM_DICE as u8 {
            Err(Self::Error::SumMismatch {
                expected: RollCounts::NUM_DICE as u8,
                got: value.iter().sum::<u8>(),
            })
        } else {
            Ok(Self(value))
        }
    }
}

impl CappedUpperSectionScore {
    pub const CAP: u8 = 63;

    pub fn score(&self) -> u8 {
        self.0
    }

    pub fn add_clamped(&self, other: u8) -> Self {
        let clamped_other = std::cmp::min(other, Self::CAP);
        Self(std::cmp::min(self.0 + clamped_other, Self::CAP))
    }
}

impl Default for CappedUpperSectionScore {
    fn default() -> Self {
        Self(0)
    }
}

impl TryFrom<u8> for CappedUpperSectionScore {
    type Error = ConstructionError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > Self::CAP {
            Err(Self::Error::ValueTooLarge {
                max: Self::CAP,
                got: value,
            })
        } else {
            Ok(Self(value))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_counts_valid_initialisation() {
        let result = RollCounts::try_from([1, 0, 3, 1, 0, 0]);
        assert!(result.is_ok());
    }

    #[test]
    fn roll_counts_invalid_initialisation() {
        let result = RollCounts::try_from([1, 1, 1, 1, 1, 1]);
        assert_eq!(
            result,
            Err(ConstructionError::SumMismatch {
                expected: RollCounts::NUM_DICE as u8,
                got: 6
            })
        );
    }

    /// If we didn't check for overflow, this would sum to 5u8, and be valid.
    #[test]
    fn roll_counts_initialisation_overflow() {
        let result = RollCounts::try_from([254, 3, 4, 0, 0, 0]);
        assert_eq!(
            result,
            Err(ConstructionError::ValueTooLarge {
                max: RollCounts::NUM_DICE as u8,
                got: 254
            })
        );
    }

    #[test]
    fn rolls_left_valid_initialisation() {
        let result = RollsLeft::try_from(2);
        assert!(result.is_ok());
    }

    #[test]
    fn rolls_left_invalid_initialisation() {
        let result = RollsLeft::try_from(3);
        assert_eq!(
            result,
            Err(ConstructionError::ValueTooLarge { max: 2, got: 3 })
        );
    }

    #[test]
    fn keep_counts_valid_initialisation() {
        let result = RollCounts::try_from([1, 0, 3, 1, 0, 0]);
        assert!(result.is_ok());
    }

    #[test]
    fn keep_counts_invalid_initialisation() {
        let result = RollCounts::try_from([1, 1, 1, 1, 1, 1]);
        assert_eq!(
            result,
            Err(ConstructionError::SumMismatch {
                expected: RollCounts::NUM_DICE as u8,
                got: 6
            })
        );
    }

    /// If we didn't check for overflow, this would sum to 5u8, and be valid.
    #[test]
    fn keep_counts_initialisation_overflow() {
        let result = RollCounts::try_from([254, 3, 4, 0, 0, 0]);
        assert_eq!(
            result,
            Err(ConstructionError::ValueTooLarge {
                max: RollCounts::NUM_DICE as u8,
                got: 254
            })
        );
    }

    #[test]
    fn capped_upper_section_score_clamped_addition() {
        let capped_score = CappedUpperSectionScore::try_from(CappedUpperSectionScore::CAP).unwrap();
        let result = capped_score.add_clamped(u8::MAX);
        assert_eq!(result.score(), CappedUpperSectionScore::CAP);
    }

    #[test]
    fn score_yahtzee() {
        let scorecard_state = ScorecardState::default()
            .score(ScoreCategory::Yahtzee, 50)
            .unwrap();
        let expected_capped_upper_section_score = 0;
        let mut expected_score_category_state =
            EnumMap::<ScoreCategory, ScoreCategoryState>::default();
        expected_score_category_state[ScoreCategory::Yahtzee] = ScoreCategoryState::Scored;
        let expected_state = ScorecardState {
            capped_upper_section_score: CappedUpperSectionScore(
                expected_capped_upper_section_score,
            ),
            score_category_state: expected_score_category_state,
        };
        assert_eq!(scorecard_state, expected_state);
    }

    #[test]
    fn scratch_yahtzee() {
        let scorecard_state = ScorecardState::default()
            .score(ScoreCategory::Yahtzee, 0)
            .unwrap();
        let expected_capped_upper_section_score = 0;
        let mut expected_score_category_state =
            EnumMap::<ScoreCategory, ScoreCategoryState>::default();
        expected_score_category_state[ScoreCategory::Yahtzee] = ScoreCategoryState::Scratched;
        let expected_state = ScorecardState {
            capped_upper_section_score: CappedUpperSectionScore(
                expected_capped_upper_section_score,
            ),
            score_category_state: expected_score_category_state,
        };
        assert_eq!(scorecard_state, expected_state);
    }

    #[test]
    fn score_upper_section() {
        let scorecard_state = ScorecardState::default()
            .score(ScoreCategory::Aces, 3)
            .unwrap();
        let expected_capped_upper_section_score = 3;
        let mut expected_score_category_state =
            EnumMap::<ScoreCategory, ScoreCategoryState>::default();
        expected_score_category_state[ScoreCategory::Aces] = ScoreCategoryState::Scored;
        let expected_state = ScorecardState {
            capped_upper_section_score: CappedUpperSectionScore(
                expected_capped_upper_section_score,
            ),
            score_category_state: expected_score_category_state,
        };
        assert_eq!(scorecard_state, expected_state);
    }

    #[test]
    fn invalid_score() {
        let scorecard_state = ScorecardState::default()
            .score(ScoreCategory::Aces, 3)
            .unwrap()
            .score(ScoreCategory::Aces, 3);
        assert_eq!(
            scorecard_state,
            Err(ScoringError::InvalidScoreCategory {
                category: ScoreCategory::Aces
            })
        );
    }

    #[test]
    fn num_faces_as_expected() {
        assert_eq!(
            RollCounts::NUM_FACES,
            6,
            "This constant was only named to reduce magic numbers in the code. It is absolutely not intended to be changed, and things will break if it is changed."
        );
    }

    #[test]
    fn num_dice_as_expected() {
        assert_eq!(
            RollCounts::NUM_DICE as u8,
            5,
            "This constant was only named to reduce magic numbers in the code. It is absolutely not intended to be changed, and things will break if it is changed."
        );
    }
}
