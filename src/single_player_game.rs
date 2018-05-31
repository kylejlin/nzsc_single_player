use super::players::{
    CharacterlessPlayer,
    BoosterlessPlayer,
    Player,
};
use super::io;
use super::prfg;

use super::moves::{
    Move,
    SINGLE_USE_MOVES,
    DESTRUCTIVE_MOVES,
};
use super::boosters::Booster;
use super::outcomes;

/// A phase of the game.
#[derive(Clone)]
pub enum Phase {
    CharacterChoosing {
        human: CharacterlessPlayer,
        computer: CharacterlessPlayer,
    },
    BoosterChoosing {
        human: BoosterlessPlayer,
        computer: BoosterlessPlayer
    },
    MoveChoosing {
        human: Player,
        computer: Player,
    },
    GameOver {
        human_points: u8,
        computer_points: u8,
    },
}

pub struct SinglePlayerNZSCGame {
    prfg: prfg::PseudorandomFloatGenerator,
    pub phase: Phase,
}

impl SinglePlayerNZSCGame {
    pub fn new(seed: u32) -> SinglePlayerNZSCGame {
        SinglePlayerNZSCGame {
            prfg: prfg::PseudorandomFloatGenerator::new(seed),
            phase: Phase::CharacterChoosing {
                human: CharacterlessPlayer::new(),
                computer: CharacterlessPlayer::new(),
            }
        }
    }

    fn generate_random_index_from_inclusive_max(&mut self, inclusive_max: usize) -> usize {
        let inclusive_max = inclusive_max as f64;

        (self.prfg.next() * (inclusive_max + 1.0)).floor() as usize
    }

    pub fn initial_output(&self) -> io::Output {
        if let Phase::CharacterChoosing { ref human, computer: _ }  = self.phase {
            io::Output {
                question: Some(io::Question::ChooseCharacter {
                    available_characters: human.available_characters(),
                }),
                notifications: vec![],
            }
        } else {
            panic!("Initial output called at wrong phase!");
        }
    }

    pub fn next(&mut self, answer: io::Answer) -> Result<io::Output, ()> {
        match (self.phase.clone(), answer) {
            (
                Phase::CharacterChoosing { mut human, mut computer },
                io::Answer::CharacterSelection(character_selection)
            ) => {
                // Closure for the sake of DRY
                let penalize_human = |waits, penalty_notification, mut human: CharacterlessPlayer, mut computer: CharacterlessPlayer, slf: &mut SinglePlayerNZSCGame| -> Result<io::Output, ()> {
                    computer.points += human.penalize_waits(waits);

                    let mut output = io::Output {
                        question: None,
                        notifications: vec![
                            penalty_notification,
                            io::Notification::ScoreUpdate {
                                human_points: human.points,
                                computer_points: computer.points,
                            }
                        ],
                    };

                    if computer.points < 5 {
                        output.question = Some(io::Question::ChooseCharacter {
                            available_characters: human.available_characters(),
                        });

                        slf.phase = Phase::CharacterChoosing {
                            human,
                            computer,
                        };
                    } else {
                        output.notifications.push(
                            io::Notification::GameOver {
                                human_points: human.points,
                                computer_points: computer.points,
                            }
                        );

                        slf.phase = Phase::GameOver {
                            human_points: human.points,
                            computer_points: computer.points,
                        };
                    }

                    Ok(output)
                };

                match character_selection {
                    io::CharacterSelection::Character(selected_human_character) => {
                        if human.available_characters().contains(&selected_human_character) {
                            let available_computer_characters = computer.available_characters();
                            let selected_computer_character = available_computer_characters[
                                self.generate_random_index_from_inclusive_max(available_computer_characters.len() - 1)
                            ];

                            if selected_human_character == selected_computer_character {
                                human.character_streak.add(selected_human_character);
                                computer.character_streak.add(selected_computer_character);

                                let available_human_characters = human.available_characters();

                                self.phase = Phase::CharacterChoosing {
                                    human,
                                    computer,
                                };

                                Ok(io::Output {
                                    question: Some(
                                        io::Question::ChooseCharacter { available_characters: available_human_characters }
                                    ),
                                    notifications: vec![
                                        io::Notification::SameCharacterSelection {
                                            both_character: selected_human_character,
                                        }
                                    ],
                                })
                            } else {
                                let headstart = outcomes::get_headstart(selected_human_character, selected_computer_character);

                                human.points += headstart.0;
                                computer.points += headstart.1;

                                let who_gets_the_headstart = match headstart {
                                    outcomes::Headstart(0, 0) => io::WhoGetsTheHeadstart::Neither,
                                    outcomes::Headstart(0, 1) => io::WhoGetsTheHeadstart::JustComputer,
                                    outcomes::Headstart(1, 0) => io::WhoGetsTheHeadstart::JustHuman,
                                    outcomes::Headstart(a, b) => panic!("Illegal headstart: {}-{}!", a, b),
                                };

                                let human = human.to_boosterless_player(selected_human_character);
                                let computer = computer.to_boosterless_player(selected_computer_character);

                                let human_character = human.character;
                                let computer_character = computer.character;


                                // Human might have incurred penalties before successfully choosing character.
                                if computer.points < 5 {
                                    self.phase = Phase::BoosterChoosing {
                                        human,
                                        computer,
                                    };

                                    Ok(io::Output {
                                        question: Some(
                                            io::Question::ChooseBooster { available_boosters: human_character.get_boosters() }
                                        ),
                                        notifications: vec![
                                            io::Notification::CharacterSelectionAndHeadstart {
                                                human_character: human_character,
                                                computer_character: computer_character,
                                                who_gets_the_headstart,
                                            }
                                        ],
                                    })
                                } else {
                                    self.phase = Phase::GameOver {
                                        human_points: human.points,
                                        computer_points: computer.points,
                                    };

                                    Ok(io::Output {
                                        question: None,
                                        notifications: vec![
                                            io::Notification::CharacterSelectionAndHeadstart {
                                                human_character: human_character,
                                                computer_character: computer_character,
                                                who_gets_the_headstart,
                                            },
                                            io::Notification::GameOver {
                                                human_points: human.points,
                                                computer_points: computer.points,
                                            }
                                        ],
                                    })
                                }
                            }
                        } else {
                            penalize_human(3, io::Notification::CharacterThreeTimesInARowPenalty {
                                attempted_character: selected_human_character,
                            }, human, computer, self)
                        }
                    },
                    io::CharacterSelection::Nonexistent(attempted_character_name) => {
                        penalize_human(4, io::Notification::CharacterNonexistentPenalty {
                            attempted_character_name,
                        }, human, computer, self)
                    },
                }
            },
            (
                Phase::BoosterChoosing { human, computer },
                io::Answer::BoosterSelection(booster_selection)
            ) => {
                // Closure for the sake of DRY
                let penalize_human = |waits, penalty_notification, mut human: BoosterlessPlayer, mut computer: BoosterlessPlayer, slf: &mut SinglePlayerNZSCGame| -> Result<io::Output, ()> {
                    computer.points += human.penalize_waits(waits);

                    let mut output = io::Output {
                        question: None,
                        notifications: vec![
                            penalty_notification,
                            io::Notification::ScoreUpdate {
                                human_points: human.points,
                                computer_points: computer.points,
                            }
                        ],
                    };

                    if computer.points < 5 {
                        output.question = Some(io::Question::ChooseBooster {
                            available_boosters: human.available_boosters(),
                        });

                        slf.phase = Phase::BoosterChoosing {
                            human,
                            computer,
                        };
                    } else {
                        output.notifications.push(
                            io::Notification::GameOver {
                                human_points: human.points,
                                computer_points: computer.points,
                            }
                        );

                        slf.phase = Phase::GameOver {
                            human_points: human.points,
                            computer_points: computer.points,
                        };
                    }

                    Ok(output)
                };

                match booster_selection {
                    io::BoosterSelection::Booster(selected_human_booster) => {
                        if human.available_boosters().contains(&selected_human_booster) {
                            let selected_computer_booster = computer.available_boosters()[
                                self.generate_random_index_from_inclusive_max(1)
                            ];
                            let human = human.to_player(selected_human_booster);
                            let computer = computer.to_player(selected_computer_booster);

                            let human_booster = human.booster;
                            let computer_booster = computer.booster;
                            let available_human_moves = human.available_moves();

                            self.phase = Phase::MoveChoosing {
                                human,
                                computer,
                            };

                            Ok(io::Output {
                                question: Some(io::Question::ChooseMove { available_moves: available_human_moves }),
                                notifications: vec![
                                    io::Notification::BoosterSelection {
                                        human_booster,
                                        computer_booster,
                                    }
                                ],
                            })
                        } else {
                            penalize_human(3, io::Notification::BoosterFromWrongCharacterPenalty {
                                attempted_booster: selected_human_booster,
                            }, human, computer, self)
                        }
                    },
                    io::BoosterSelection::Nonexistent(attempted_booster_name) => {
                        penalize_human(4, io::Notification::BoosterNonexistentPenalty {
                            attempted_booster_name,
                        }, human, computer, self)
                    },
                }
            },
            (
                Phase::MoveChoosing { mut human, mut computer },
                io::Answer::MoveSelection(move_selection)
            ) => {
                // Closure for the sake of DRY
                let penalize_human = |waits, penalty_notification, mut human: Player, mut computer: Player, slf: &mut SinglePlayerNZSCGame| -> Result<io::Output, ()> {
                    computer.points += human.penalize_waits(waits);

                    let mut output = io::Output {
                        question: None,
                        notifications: vec![
                            penalty_notification,
                            io::Notification::ScoreUpdate {
                                human_points: human.points,
                                computer_points: computer.points,
                            }
                        ],
                    };

                    if computer.points < 5 {
                        output.question = Some(io::Question::ChooseMove {
                            available_moves: human.available_moves(),
                        });

                        slf.phase = Phase::MoveChoosing {
                            human,
                            computer,
                        };
                    } else {
                        output.notifications.push(
                            io::Notification::GameOver {
                                human_points: human.points,
                                computer_points: computer.points,
                            }
                        );

                        slf.phase = Phase::GameOver {
                            human_points: human.points,
                            computer_points: computer.points,
                        };
                    }

                    Ok(output)
                };

                match move_selection {
                    io::MoveSelection::Move(selected_human_move) => {
                        if human.available_moves().contains(&selected_human_move) {
                            let available_computer_moves = computer.available_moves();
                            let selected_computer_move = available_computer_moves[
                                self.generate_random_index_from_inclusive_max(available_computer_moves.len() - 1)
                            ];

                            human.move_streak.add(selected_human_move);
                            computer.move_streak.add(selected_computer_move);

                            if SINGLE_USE_MOVES.contains(&selected_human_move)
                                || DESTRUCTIVE_MOVES.contains(&selected_computer_move)
                            {
                                human.destroyed_moves.push(selected_human_move);
                            }
                            if SINGLE_USE_MOVES.contains(&selected_computer_move)
                                || DESTRUCTIVE_MOVES.contains(&selected_human_move)
                            {
                                computer.destroyed_moves.push(selected_computer_move);
                            }

                            let mut points = outcomes::get_points(vec![selected_human_move, selected_computer_move]);

                            if selected_human_move == Move::ShadowFireball && selected_computer_move == Move::Smash {
                                if computer.booster == Booster::Strong {
                                    points[0] = 0;
                                    points[1] = 1;
                                } else {
                                    points[0] = 1;
                                    points[1] = 0;
                                }
                            } else if selected_human_move == Move::Smash && selected_computer_move == Move::ShadowFireball {
                                if human.booster == Booster::Strong {
                                    points[0] = 1;
                                    points[1] = 0;
                                } else {
                                    points[0] = 0;
                                    points[1] = 1;
                                }
                            }

                            human.points += points[0];
                            computer.points += points[1];

                            let who_gets_the_point = match (points[0], points[1]) {
                                (0, 0) => io::WhoGetsThePoint::Neither,
                                (0, 1) => io::WhoGetsThePoint::JustComputer,
                                (1, 0) => io::WhoGetsThePoint::JustHuman,
                                (1, 1) => io::WhoGetsThePoint::Both,
                                (a, b) => panic!("Illegal outcome: {}-{}!", a, b),
                            };

                            let mut output = io::Output {
                                question: None,
                                notifications: vec![
                                    io::Notification::MoveSelectionAndOutcome {
                                        human_move: selected_human_move,
                                        computer_move: selected_computer_move,
                                        who_gets_the_point,
                                    },
                                    io::Notification::ScoreUpdate {
                                        human_points: human.points,
                                        computer_points: computer.points,
                                    }
                                ],
                            };

                            if human.points >= 5 || computer.points >= 5 {
                                if human.points == computer.points {
                                    output.question = Some(io::Question::ChooseMove {
                                        available_moves: human.available_moves(),
                                    });

                                    output.notifications.push(
                                        io::Notification::TiebreakingScoreSetback {
                                            both_points: human.points,
                                        }
                                    );

                                    human.points = 4;
                                    computer.points = 4;

                                    self.phase = Phase::MoveChoosing {
                                        human,
                                        computer,
                                    };
                                } else {
                                    output.notifications.push(
                                        io::Notification::GameOver {
                                            human_points: human.points,
                                            computer_points: computer.points,
                                        }
                                    );

                                    self.phase = Phase::GameOver {
                                        human_points: human.points,
                                        computer_points: computer.points,
                                    };
                                }
                            } else {
                                output.question = Some(io::Question::ChooseMove {
                                    available_moves: human.available_moves(),
                                });

                                self.phase = Phase::MoveChoosing {
                                    human,
                                    computer,
                                };
                            }

                            Ok(output)
                        } else {
                            if human.destroyed_moves.contains(&selected_human_move) {
                                if SINGLE_USE_MOVES.contains(&selected_human_move) {
                                    penalize_human(4, io::Notification::MoveSingleUsePenalty {
                                        attempted_move: selected_human_move,
                                    }, human, computer, self)
                                } else {
                                    penalize_human(4, io::Notification::MoveDestroyedPenalty {
                                        attempted_move: selected_human_move,
                                    }, human, computer, self)
                                }
                            } else if human.move_streak.times == 3 && human.move_streak.repeated_move == Some(selected_human_move) {
                                penalize_human(3, io::Notification::MoveThreeTimesInARowPenalty {
                                    attempted_move: selected_human_move,
                                }, human, computer, self)
                            } else {
                                let mut booster_moves = vec![];
                                for booster in &human.character.get_boosters() {
                                    booster_moves.extend(booster.get_moves());
                                }

                                if booster_moves.contains(&selected_human_move) {
                                    penalize_human(2, io::Notification::MoveFromWrongBoosterPenalty {
                                        attempted_move: selected_human_move,
                                    }, human, computer, self)
                                } else {
                                    penalize_human(3, io::Notification::MoveFromWrongCharacterPenalty {
                                        attempted_move: selected_human_move,
                                    }, human, computer, self)
                                }
                            }
                        }
                    },
                    io::MoveSelection::Nonexistent(attempted_move_name) => {
                        penalize_human(4, io::Notification::MoveNonexistentPenalty {
                            attempted_move_name,
                        }, human, computer, self)
                    },
                }
            },
            _ => {
                Err(())
            },
        }
    }
}
