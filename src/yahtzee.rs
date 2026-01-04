use crate::types::{
    CappedUpperSectionScore, JokerRule, RollCounts, ScoreCategory, ScoreCategoryState,
    ScorecardState, ScoringError,
};
use enum_map::EnumMap;
use strum::IntoEnumIterator;

const YAHTZEE_BONUS_VALUE: u8 = 100;
const UPPER_SECTION_BONUS_VALUE: u8 = 35;

impl ScorecardState {
    /// All score categories that can be chosen for the given roll and joker rule.
    pub fn valid_score_categories(
        &self,
        roll: &RollCounts,
        joker_rule: JokerRule,
    ) -> Vec<ScoreCategory> {
        if let Some(yahtzee_category) = roll.is_yahtzee()
            && joker_rule == JokerRule::Forced
            && self.score_category_state()[yahtzee_category] == ScoreCategoryState::Unscored
        {
            return vec![yahtzee_category];
        }
        // aside from this special case, every roll is scorable in every category. It may just
        // score 0 points.
        ScoreCategory::iter()
            .filter(|&x| self.score_category_state()[x] == ScoreCategoryState::Unscored)
            .collect()
    }

    /// All valid score categories if the roll is not a yahtzee.
    pub fn valid_non_yahtzee_score_categories(&self) -> Vec<ScoreCategory> {
        ScoreCategory::iter()
            .filter(|&x| self.score_category_state()[x] == ScoreCategoryState::Unscored)
            .collect()
    }

    /// Returns (category_score, bonus_score) where category score is the base score, and bonus
    /// score is additional score from either scoring a bonus yahtzee or completing the upper
    /// section.
    pub fn score_value(
        &self,
        roll: &RollCounts,
        category: ScoreCategory,
        joker_rule: JokerRule,
    ) -> Result<(u8, u8), ScoringError> {
        // test whether category is valid
        if self.score_category_state()[category] != ScoreCategoryState::Unscored {
            return Err(ScoringError::InvalidScoreCategory { category: category });
        }
        let is_yahtzee = roll.is_yahtzee();
        if let Some(yahtzee_category) = is_yahtzee
            && joker_rule == JokerRule::Forced
            && self.score_category_state()[yahtzee_category] == ScoreCategoryState::Unscored
        {
            return Err(ScoringError::InvalidScoreCategory { category: category });
        }
        let category_score = roll.score_value(category, self.is_joker(roll));
        let yahtzee_bonus = if let Some(_) = is_yahtzee
            && self.score_category_state()[ScoreCategory::Yahtzee] == ScoreCategoryState::Scored
        {
            YAHTZEE_BONUS_VALUE
        } else {
            0
        };
        let upper_section_bonus = if category.is_upper_section()
            && self.capped_upper_section_score.score() + category_score
                >= CappedUpperSectionScore::CAP
        {
            UPPER_SECTION_BONUS_VALUE
        } else {
            0
        };
        Ok((category_score, yahtzee_bonus + upper_section_bonus))
    }

    /// Whether this state is a terminal state, i.e. if the game is ended at this state.
    pub fn is_terminal(&self) -> bool {
        self.score_category_state()
            .values()
            .all(|&v| v != ScoreCategoryState::Unscored)
    }

    fn is_joker(&self, roll_counts: &RollCounts) -> bool {
        if let Some(yahtzee_category) = roll_counts.is_yahtzee()
            && self.score_category_state()[yahtzee_category] != ScoreCategoryState::Unscored
        {
            true
        } else {
            false
        }
    }
}

impl RollCounts {
    /// Returns None if not a yahtzee, or the corresponding upper-section score category if a
    /// yahtzee.
    pub fn is_yahtzee(&self) -> Option<ScoreCategory> {
        if !self
            .roll_counts()
            .iter()
            .any(|&x| x as usize == RollCounts::NUM_DICE)
        {
            return None;
        }
        const VAL: u8 = RollCounts::NUM_DICE as u8;
        match &self.roll_counts() {
            [VAL, 0, 0, 0, 0, 0] => Some(ScoreCategory::Aces),
            [0, VAL, 0, 0, 0, 0] => Some(ScoreCategory::Twos),
            [0, 0, VAL, 0, 0, 0] => Some(ScoreCategory::Threes),
            [0, 0, 0, VAL, 0, 0] => Some(ScoreCategory::Fours),
            [0, 0, 0, 0, VAL, 0] => Some(ScoreCategory::Fives),
            [0, 0, 0, 0, 0, VAL] => Some(ScoreCategory::Sixes),
            _ => unreachable!(),
        }
    }

    /// Returns the value of scoring this roll as the input category.
    pub fn score_value(&self, category: ScoreCategory, is_joker: bool) -> u8 {
        use ScoreCategory::*;

        let roll_counts = self.roll_counts();

        match category {
            Aces => roll_counts[0],
            Twos => roll_counts[1] * 2,
            Threes => roll_counts[2] * 3,
            Fours => roll_counts[3] * 4,
            Fives => roll_counts[4] * 5,
            Sixes => roll_counts[5] * 6,
            FullHouse => {
                if is_joker
                    || (roll_counts.iter().any(|&x| x == 3) && roll_counts.iter().any(|&x| x == 2))
                {
                    25
                } else {
                    0
                }
            }
            ThreeOfAKind => {
                if roll_counts.iter().any(|&x| x >= 3) {
                    self.sum()
                } else {
                    0
                }
            }
            FourOfAKind => {
                if roll_counts.iter().any(|&x| x >= 4) {
                    self.sum()
                } else {
                    0
                }
            }
            SmallStraight => {
                if is_joker || self.straight_length() >= 4 {
                    30
                } else {
                    0
                }
            }
            LargeStraight => {
                if is_joker || self.straight_length() >= 5 {
                    40
                } else {
                    0
                }
            }
            Yahtzee => {
                if let Some(_) = self.is_yahtzee() {
                    50
                } else {
                    0
                }
            }
            Chance => self.sum(),
        }
    }

    fn sum(&self) -> u8 {
        self.roll_counts()
            .iter()
            .zip(1u8..=Self::NUM_FACES as u8)
            .map(|(&x, y)| x * y)
            .sum()
    }

    fn straight_length(&self) -> u8 {
        let mut max: u8 = 0;
        let mut cur: u8 = 0;
        for &count in self.roll_counts() {
            if count > 0 {
                cur += 1;
            }
            max = std::cmp::max(cur, max);
        }
        max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_categories_free_choice_joker_rule() {
        let scorecard_state = ScorecardState::default()
            .score(ScoreCategory::Aces, 3)
            .unwrap();
        let roll = RollCounts::try_from([5, 0, 0, 0, 0, 0]).unwrap();
        let joker_rule = JokerRule::FreeChoice;
        let valid_categories = scorecard_state.valid_score_categories(&roll, joker_rule);
        let mut valid_categories_map = EnumMap::<ScoreCategory, bool>::default();
        for category in ScoreCategory::iter() {
            valid_categories_map[category] = false;
        }
        for category in valid_categories {
            valid_categories_map[category] = true;
        }
        for (k, &v) in &valid_categories_map {
            match k {
                ScoreCategory::Aces => assert!(!v, "Expected {:#?} to be false.", k),
                _ => assert!(v, "Expected {:#?} to be true.", k),
            };
        }
    }

    #[test]
    fn valid_categories_forced_joker_rule() {
        let scorecard_state = ScorecardState::default()
            .score(ScoreCategory::Yahtzee, 50)
            .unwrap();
        let roll = RollCounts::try_from([5, 0, 0, 0, 0, 0]).unwrap();
        let joker_rule = JokerRule::Forced;
        let valid_categories = scorecard_state.valid_score_categories(&roll, joker_rule);
        let mut valid_categories_map = EnumMap::<ScoreCategory, bool>::default();
        for category in ScoreCategory::iter() {
            valid_categories_map[category] = false;
        }
        for category in valid_categories {
            valid_categories_map[category] = true;
        }
        for (k, &v) in &valid_categories_map {
            match k {
                ScoreCategory::Aces => assert!(v, "Expected {:#?} to be true.", k),
                _ => assert!(!v, "Expected {:#?} to be false.", k),
            }
        }
    }

    #[test]
    fn test_yahtzee_is_yahtzee() {
        let roll_counts = RollCounts::try_from([0, 5, 0, 0, 0, 0]).unwrap();
        assert_eq!(roll_counts.is_yahtzee(), Some(ScoreCategory::Twos));
    }

    #[test]
    fn test_not_yahtzee_is_not_yahtzee() {
        let roll_counts = RollCounts::try_from([1, 1, 1, 1, 1, 0]).unwrap();
        assert_eq!(roll_counts.is_yahtzee(), None);
    }

    #[test]
    fn test_upper_category_score() {
        let roll_counts = RollCounts::try_from([2, 2, 1, 0, 0, 0]).unwrap();
        let expected = 4;
        let score = roll_counts.score_value(ScoreCategory::Twos, false);
        assert_eq!(expected, score);
    }

    #[test]
    fn test_yahtzee_score() {
        let roll_counts = RollCounts::try_from([0, 0, 5, 0, 0, 0]).unwrap();
        let expected = 50;
        let score = roll_counts.score_value(ScoreCategory::Yahtzee, false);
        assert_eq!(expected, score);
    }

    #[test]
    fn test_not_yahtzee_score() {
        let roll_counts = RollCounts::try_from([0, 0, 4, 0, 1, 0]).unwrap();
        let expected = 0;
        let score = roll_counts.score_value(ScoreCategory::Yahtzee, false);
        assert_eq!(expected, score);
    }

    #[test]
    fn test_small_straight_score() {
        let roll_counts = RollCounts::try_from([0, 2, 1, 1, 1, 0]).unwrap();
        let expected = 30;
        let score = roll_counts.score_value(ScoreCategory::SmallStraight, false);
        assert_eq!(expected, score);
    }

    #[test]
    fn test_yahtzee_scored_as_full_house_no_joker() {
        let roll_counts = RollCounts::try_from([0, 5, 0, 0, 0, 0]).unwrap();
        let expected = 0;
        let score = roll_counts.score_value(ScoreCategory::FullHouse, false);
        assert_eq!(expected, score);
    }

    #[test]
    fn test_yahtzee_scored_as_full_house_yes_joker() {
        let roll_counts = RollCounts::try_from([0, 5, 0, 0, 0, 0]).unwrap();
        let expected = 25;
        let score = roll_counts.score_value(ScoreCategory::FullHouse, true);
        assert_eq!(expected, score);
    }

    #[test]
    fn test_terminal() {
        let mut scorecard_state = ScorecardState::default();
        for score_category in ScoreCategory::iter() {
            scorecard_state = scorecard_state.score(score_category, 0).unwrap();
        }
        let is_terminal = scorecard_state.is_terminal();
        assert!(is_terminal);
    }

    #[test]
    fn test_non_terminal() {
        let scorecard_state = ScorecardState::default()
            .score(ScoreCategory::Aces, 3)
            .unwrap();
        let is_terminal = scorecard_state.is_terminal();
        assert!(!is_terminal);
    }
}
