use super::super::characters::Character;
use super::super::boosters::Booster;
use super::super::moves::Move;

/// The answer to a `Question`.
pub enum Answer {
    CharacterSelection(CharacterSelection),
    BoosterSelection(BoosterSelection),
    MoveSelection(MoveSelection),
}

pub enum CharacterSelection {
    Character(Character),
    Nonexistent(String),
}

pub enum BoosterSelection {
    Booster(Booster),
    Nonexistent(String),
}

pub enum MoveSelection {
    Move(Move),
    Nonexistent(String),
}
