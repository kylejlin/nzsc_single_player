use super::moves::Move;
use super::characters::Character;

/// Keeps track of how many times you chose a move in a row.
///
/// Used for enforcing three-times-in-a-row rule.
#[derive(Clone, Copy)]
pub struct MoveStreak {
    pub repeated_move: Option<Move>,
    pub times: u8
}

/// Keeps track of how many times you chose a character in a row.
///
/// Used for enforcing three-times-in-a-row rule.
#[derive(Clone, Copy)]
pub struct CharacterStreak {
    pub repeated_character: Option<Character>,
    pub times: u8
}

impl MoveStreak {
    pub fn new() -> Self {
        Self {
            repeated_move: None,
            times: 0
        }
    }

    pub fn add(&mut self, new_move: Move) {
        let is_streak_continued = if let Some(repeated_move) = self.repeated_move {
            repeated_move == new_move
        } else {
            false
        };

        self.repeated_move = Some(new_move);
        self.times = if is_streak_continued {
            self.times + 1
        } else {
            1
        }
    }
}

impl CharacterStreak {
    pub fn new() -> Self {
        Self {
            repeated_character: None,
            times: 0
        }
    }

    pub fn add(&mut self, new_character: Character) {
        let is_streak_continued = if let Some(repeated_character) = self.repeated_character {
            repeated_character == new_character
        } else {
            false
        };

        self.repeated_character = Some(new_character);
        self.times = if is_streak_continued {
            self.times + 1
        } else {
            1
        }
    }
}
