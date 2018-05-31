use super::super::characters::Character;
use super::super::boosters::Booster;
use super::super::moves::Move;

/// A request to the user for an `Answer`.
///
/// This is how input is obtained.
/// Every question is associated with a context (information that helps the user in answering the question).
/// The context is stored in the variant fields.
pub enum Question {
    ChooseCharacter {
        available_characters: Vec<Character>,
    },
    ChooseBooster {
        available_boosters: Vec<Booster>,
    },
    ChooseMove {
        available_moves: Vec<Move>
    },
}
