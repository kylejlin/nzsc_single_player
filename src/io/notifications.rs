use super::super::characters::Character;
use super::super::boosters::Booster;
use super::super::moves::Move;

/// Something the user should know, but doesn't need to answer.
pub enum Notification {
    CharacterSelectionAndHeadstart {
        human_character: Character,
        computer_character: Character,
        who_gets_the_headstart: WhoGetsTheHeadstart,
    },
    SameCharacterSelection {
        both_character: Character,
    },

    BoosterSelection {
        human_booster: Booster,
        computer_booster: Booster,
    },

    MoveSelectionAndOutcome {
        human_move: Move,
        computer_move: Move,
        who_gets_the_point: WhoGetsThePoint,
    },

    ScoreUpdate {
        human_points: u8,
        computer_points: u8,
    },
    TiebreakingScoreSetback {
        both_points: u8,
    },
    GameOver {
        human_points: u8,
        computer_points: u8,
    },

    CharacterNonexistentPenalty {
        attempted_character_name: String,
    },
    CharacterThreeTimesInARowPenalty {
        attempted_character: Character,
    },

    BoosterNonexistentPenalty {
        attempted_booster_name: String,
    },
    BoosterFromWrongCharacterPenalty {
        attempted_booster: Booster,
    },

    MoveNonexistentPenalty {
        attempted_move_name: String,
    },
    MoveThreeTimesInARowPenalty {
        attempted_move: Move,
    },
    MoveSingleUsePenalty {
        attempted_move: Move,
    },
    MoveDestroyedPenalty {
        attempted_move: Move,
    },
    MoveFromWrongCharacterPenalty {
        attempted_move: Move,
    },
    MoveFromWrongBoosterPenalty {
        attempted_move: Move,
    },
}

pub enum WhoGetsThePoint {
    Neither,
    JustComputer,
    JustHuman,
    Both,
}

pub enum WhoGetsTheHeadstart {
    Neither,
    JustComputer,
    JustHuman,
}
