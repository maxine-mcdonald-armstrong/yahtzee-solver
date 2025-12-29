use rand::Rng;

pub const YAHTZEE_BONUS_VALUE: u16 = 50;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct RollCounts([u8; 6]);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum RollError {
    InvalidNumberOfDice(u16),
    InvalidReroll,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum SubtractionError {
    ResultLessThanZero,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct RollsLeft(u8);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RollsLeftCreationError {
    RollsLeftOutOfBounds,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DiceState {
    roll_counts: RollCounts,
    rolls_left: RollsLeft,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct CappedUpperSectionSum(u8);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum CappedUpperSectionSumCreationError {
    SumOutOfBounds,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub enum YahtzeeScoreState {
    #[default]
    Unscored,
    Scratched,
    Scored,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub enum NonYahtzeeScoreState {
    #[default]
    Unscored,
    Scored,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct ScorecardState {
    pub upper_section_sum: CappedUpperSectionSum,
    pub non_yahtzee_scored: [NonYahtzeeScoreState; 12],
    pub yahtzee_state: YahtzeeScoreState,
}

impl TryFrom<[u8; 6]> for RollCounts {
    type Error = RollError;

    fn try_from(counts: [u8; 6]) -> Result<Self, Self::Error> {
        let n_dice: u16 = counts.iter().map(|&a| a as u16).sum();
        if n_dice != 5 {
            return Err(Self::Error::InvalidNumberOfDice(n_dice));
        }
        Ok(Self(counts))
    }
}

impl RollCounts {
    pub fn roll_counts(&self) -> [u8; 6] {
        self.0
    }

    pub fn roll<R: Rng + ?Sized>(rng: &mut R) -> Self {
        let mut counts = [0u8; 6];
        for _ in 0..5 {
            let die = rng.random_range(0..=5);
            counts[die] += 1;
        }
        RollCounts::try_from(counts)
            .expect("Our counts should add to exactly 5, so this should not fail.")
    }

    pub fn reroll<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        to_reroll: [u8; 6],
    ) -> Result<Self, RollError> {
        let mut counts = self
            .subtract(to_reroll)
            .map_err(|_| RollError::InvalidReroll)?;
        let n_to_reroll = to_reroll.iter().sum();
        for _ in 0..n_to_reroll {
            let die = rng.random_range(0..=5);
            counts[die] += 1;
        }
        Ok(RollCounts::try_from(counts)
            .expect("Our counts should add to exactly 5, so this should not fail."))
    }

    fn subtract(&self, to_subtract: [u8; 6]) -> Result<[u8; 6], SubtractionError> {
        let base: [u8; 6] = self.roll_counts();
        let mut out = [0u8; 6];

        for i in 0..6 {
            out[i] = base[i]
                .checked_sub(to_subtract[i])
                .ok_or(SubtractionError::ResultLessThanZero)?;
        }

        Ok(out)
    }
}

impl TryFrom<u8> for RollsLeft {
    type Error = RollsLeftCreationError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 2 {
            return Err(Self::Error::RollsLeftOutOfBounds);
        }
        Ok(Self(value))
    }
}

impl Default for RollsLeft {
    fn default() -> Self {
        Self(2)
    }
}

impl RollsLeft {
    pub fn rolls_left(&self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for CappedUpperSectionSum {
    type Error = CappedUpperSectionSumCreationError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 63 {
            return Err(Self::Error::SumOutOfBounds);
        }
        Ok(Self(value))
    }
}

impl CappedUpperSectionSum {
    pub fn capped_upper_section_sum(&self) -> u8 {
        self.0
    }
}
