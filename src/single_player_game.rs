use super::moves::Move;
use super::boosters::Booster;
use super::characters::Character;
use super::outcomes;
use super::players::Player;

use super::prfg;

use super::command_line_app;

use std::str::FromStr;

const SINGLE_USE_MOVES: [Move; 3] = [
    Move::Zap,
    Move::Regenerate,
    Move::AcidSpray
];
const DESTRUCTIVE_MOVES: [Move; 2] = [
    Move::Zap,
    Move::AcidSpray
];

fn get_victory_term_by_margin(margin: u8) -> String {
    match margin {
        1 => "Clinch".to_string(),
        2 => "Hypnotization".to_string(),
        3 => "Obliteration".to_string(),
        4 => "Annihilation".to_string(),
        5 => "Wipeout".to_string(),
        _ => {
            panic!("Impossible victory margin: {}", margin);
        }
    }
}

pub struct SinglePlayerNZSCGame {
    human: Player,
    computer: Player,
    prfg: prfg::PseudorandomFloatGenerator,
}

impl SinglePlayerNZSCGame {
    pub fn new(seed: u32) -> SinglePlayerNZSCGame {
        SinglePlayerNZSCGame {
            human: Player::new(),
            computer: Player::new(),
            prfg: prfg::PseudorandomFloatGenerator::new(seed),
        }
    }

    fn generate_random_index_from_inclusive_max(&mut self, inclusive_max: usize) -> usize {
        let inclusive_max = inclusive_max as f64;

        (self.prfg.next() * (inclusive_max + 1.0)).floor() as usize
    }
}

impl command_line_app::CommandLineApp for SinglePlayerNZSCGame {
    fn initial_prompt(&self) -> String {
        "Choose a character:\n\tNinja\n\tZombie\n\tSamurai\n\tClown\n".to_string()
    }

    fn next(&mut self, response: String) -> command_line_app::Prompt {
        let mut output = String::new();

        if let Some(human_booster) = self.human.booster {
            let computer_booster = self.computer.booster.expect("Impossible state: Human has booster but not computer.");
            if let Ok(selected_human_move) = Move::from_str(&response[..]) {
                if self.human.available_moves().contains(&selected_human_move) {
                    let available_computer_moves = self.computer.available_moves();
                    let selected_computer_move = available_computer_moves[
                        self.generate_random_index_from_inclusive_max(available_computer_moves.len() - 1)
                    ];

                    output = format!("You chose {}. Computer chose {}.\n", selected_human_move, selected_computer_move);

                    self.human.move_streak.update(selected_human_move);
                    self.computer.move_streak.update(selected_computer_move);

                    if SINGLE_USE_MOVES.contains(&selected_human_move)
                        || DESTRUCTIVE_MOVES.contains(&selected_computer_move)
                    {
                        self.human.exhausted_moves.push(selected_human_move);
                    }
                    if SINGLE_USE_MOVES.contains(&selected_computer_move)
                        || DESTRUCTIVE_MOVES.contains(&selected_human_move)
                    {
                        self.computer.exhausted_moves.push(selected_computer_move);
                    }

                    let mut points = outcomes::get_points(vec![selected_human_move, selected_computer_move]);

                    if selected_human_move == Move::ShadowFireball && selected_computer_move == Move::Smash {
                        if computer_booster == Booster::Strong {
                            points[0] = 0;
                            points[1] = 1;
                        } else {
                            points[0] = 1;
                            points[1] = 0;
                        }
                    } else if selected_human_move == Move::Smash && selected_computer_move == Move::ShadowFireball {
                        if human_booster == Booster::Strong {
                            points[0] = 1;
                            points[1] = 0;
                        } else {
                            points[0] = 0;
                            points[1] = 1;
                        }
                    }

                    self.human.points += points[0];
                    self.computer.points += points[1];

                    let outcome_message = match (points[0], points[1]) {
                        (0, 0) => "As a result, neither of you gets a point.\n",
                        (0, 1) => "As a result, the computer gets a point.\n",
                        (1, 0) => "As a result, you get a point.\n",
                        (1, 1) => "As a result, both of you get a point.\n",
                        _ => panic!("Impossible state: Impossible move vs. move outcome."),
                    };
                    output.push_str(outcome_message);

                    output.push_str(
                        &format!("The score is now {}-{}.\n\n", self.human.points, self.computer.points)[..]
                    );

                    if self.human.points == self.computer.points
                        && self.human.points >= 5
                    {
                        self.human.points = 4;
                        self.computer.points = 4;
                        output.push_str("Since there is a tie, the score will be reset to 4-4.\n\n");
                    }

                    if self.human.points < 5 && self.computer.points < 5 {
                        output.push_str("Choose a move:\n");
                        for available_move in &self.human.available_moves() {
                            output.push_str(
                                &format!("\t{}\n", available_move)[..]
                            );
                        }
                    } else {
                        let game_over_message = if self.human.points > self.computer.points {
                            format!("You won {}-{} ({}).\n", self.human.points, self.computer.points, get_victory_term_by_margin(self.human.points - self.computer.points))
                        } else {
                            format!("You lost {}-{} ({}).\n", self.human.points, self.computer.points, get_victory_term_by_margin(self.computer.points - self.human.points))
                        };
                        output.push_str(&game_over_message[..]);
                        return command_line_app::Prompt {
                            text: output,
                            is_final: true,
                        };
                    }
                } else {
                    let mut human_booster_moves: Vec<Move> = vec![];
                    for booster in &self.human.character.expect("Impossible state: Human has booster but no character").get_boosters() {
                        human_booster_moves.extend(booster.get_moves());
                    }

                    // NZSC Rule 3.4.1
                    if self.human.exhausted_moves.contains(&selected_human_move) {
                        let penalty_message = if SINGLE_USE_MOVES.contains(&selected_human_move) {
                            format!("{} is single-use. You cannot use it again.", selected_human_move)
                        } else {
                            format!("{} has been destroyed. You cannot use it anymore.", selected_human_move)
                        };
                        self.computer.points += self.human.penalize_waits(4);
                        output = format!("{} 4 wait penalty!\nThe score is now {}-{}.\n", penalty_message, self.human.points, self.computer.points);
                    }
                    // NZSC Rule 3.4.2
                    else if self.human.move_streak.repeated_move == Some(selected_human_move)
                        && self.human.move_streak.times == 3
                    {
                        self.computer.points += self.human.penalize_waits(3);
                        output = format!("You have already chosen {} 3 times in a row. You must choose something else before choosing it again. 3 wait penalty!\nThe score is now {}-{}.\n", selected_human_move, self.human.points, self.computer.points);
                    }
                    // NZSC Rule 3.4.3
                    else if human_booster_moves.contains(&selected_human_move) {
                        self.computer.points += self.human.penalize_waits(2);
                        output = format!("{} is from the wrong booster. 2 wait penalty!\nThe score is now {}-{}.\n", selected_human_move, self.human.points, self.computer.points);
                    }
                    // NZSC Rule 3.4.4
                    else {
                        self.computer.points += self.human.penalize_waits(3);
                        output = format!("{} is from the wrong character. 3 wait penalty!\nThe score is now {}-{}.\n\n", selected_human_move, self.human.points, self.computer.points);
                    }

                    if self.computer.points < 5 {
                        output.push_str("Choose a move:\n");
                        for available_move in &self.human.available_moves() {
                            output.push_str(
                                &format!("\t{}\n", available_move)[..]
                            );
                        }
                    } else {
                        output.push_str(
                            &format!("You lost {}-{} ({}).\n", self.human.points, self.computer.points, get_victory_term_by_margin(self.computer.points - self.human.points))[..]
                        );

                        return command_line_app::Prompt {
                            text: output,
                            is_final: true,
                        };
                    }
                }
            } else {
                self.computer.points += self.human.penalize_waits(4);
                output = format!("\"{}\" is not a move. 4 wait penalty!\nThe score is now {}-{}.\n\n", response, self.human.points, self.computer.points);

                if self.computer.points < 5 {
                    output.push_str("Choose a move:\n");
                    for available_move in &self.human.available_moves() {
                        output.push_str(
                            &format!("\t{}\n", available_move)[..]
                        );
                    }
                } else {
                    output.push_str(
                        &format!("You lost {}-{} ({}).\n", self.human.points, self.computer.points, get_victory_term_by_margin(self.computer.points - self.human.points))[..]
                    );

                    return command_line_app::Prompt {
                        text: output,
                        is_final: true,
                    };
                }
            }
        } else if let Some(human_character) = self.human.character {
            let computer_character = self.computer.character.expect("Impossible state: Human has character but not computer.");
            if let Ok(selected_human_booster) = Booster::from_str(&response[..]) {
                if human_character.get_boosters().contains(&selected_human_booster) {
                    let selected_computer_booster = computer_character.get_boosters()[self.generate_random_index_from_inclusive_max(1)];
                    self.human.booster = Some(selected_human_booster);
                    self.computer.booster = Some(selected_computer_booster);

                    output = format!("You chose {}.\nComputer chose {}.\nLet the battle begin!\n\nChoose a move:\n", selected_human_booster, selected_computer_booster);
                    for available_move in &self.human.available_moves() {
                        output.push_str(
                            &format!("\t{}\n", available_move)[..]
                        );
                    }
                } else {
                    self.computer.points += self.human.penalize_waits(3);
                    output.push_str(
                        &format!("{} is from the wrong character. 3 wait penalty!\nThe score is now {}-{}.\n\n", selected_human_booster, self.human.points, self.computer.points)[..]
                    );

                    if self.computer.points < 5 {
                        output.push_str("Choose a booster:\n");
                        for booster in &human_character.get_boosters() {
                            output.push_str(
                                &format!("\t{}\n", booster)[..]
                            );
                        }
                    } else {
                        output.push_str(
                            &format!("You lost {}-{} ({}).\n", self.human.points, self.computer.points, get_victory_term_by_margin(self.computer.points - self.human.points))[..]
                        );

                        return command_line_app::Prompt {
                            text: output,
                            is_final: true,
                        };
                    }
                }
            } else {
                self.computer.points += self.human.penalize_waits(3);
                output.push_str(
                    &format!("\"{}\" is not a booster. 3 wait penalty!\nThe score is now {}-{}.\n\n", response, self.human.points, self.computer.points)[..]
                );

                if self.computer.points < 5 {
                    output.push_str("Choose a booster:\n");
                    for booster in &human_character.get_boosters() {
                        output.push_str(
                            &format!("\t{}\n", booster)[..]
                        );
                    }
                } else {
                    output.push_str(
                        &format!("You lost {}-{} ({}).\n", self.human.points, self.computer.points, get_victory_term_by_margin(self.computer.points - self.human.points))[..]
                    );

                    return command_line_app::Prompt {
                        text: output,
                        is_final: true,
                    };
                }
            }
        } else {
            if let Ok(selected_human_character) = Character::from_str(&response[..]) {
                let selected_computer_character = [
                    Character::Ninja,
                    Character::Zombie,
                    Character::Samurai,
                    Character::Clown
                ][self.generate_random_index_from_inclusive_max(3)];
                if selected_human_character == selected_computer_character {
                    self.human.character_streak.update(selected_human_character);
                    self.computer.character_streak.update(selected_computer_character);
                    output.push_str(
                        &format!("\nBoth of you chose {0}, so you must repick.\nYou have picked {0} {1} times.\nComputer has picked {0} {2} times.\n\n", selected_human_character, self.human.character_streak.times, self.computer.character_streak.times)[..]
                    );

                    let mut available_human_characters = vec![
                        Character::Ninja,
                        Character::Zombie,
                        Character::Samurai,
                        Character::Clown
                    ];
                    if self.human.character_streak.times == 3 {
                        available_human_characters.retain(|&c| {
                            Some(c) != self.human.character_streak.repeated_character
                        });
                    }

                    output.push_str("Choose a character:\n");
                    for character in &available_human_characters {
                        output.push_str(
                            &format!("\t{}\n", character)
                        );
                    }
                } else {
                    self.human.character = Some(selected_human_character);
                    self.computer.character = Some(selected_computer_character);

                    output.push_str(
                        &format!("\nYou chose {}.\nComputer chose {}.\n", selected_human_character, selected_computer_character)[..]
                    );

                    let headstart = outcomes::get_headstart(selected_human_character, selected_computer_character);
                    self.human.points += headstart.0;
                    self.human.points += headstart.1;

                    let headstart_message = match headstart {
                        outcomes::Headstart(0, 0) => "As a result, neither of you gets a headstart.\n",
                        outcomes::Headstart(0, 1) => "As a result, the computer gets a headstart.\n",
                        outcomes::Headstart(1, 0) => "As a result, you get a headstart.\n",
                        _ => panic!("Impossible state: More than one character has a headstart!"),
                    };
                    output.push_str(headstart_message);
                    output.push_str(
                        &format!("The score is now {}-{}.\n\n", self.human.points, self.computer.points)[..]
                    );

                    if self.computer.points < 5 {
                        output.push_str("Choose a booster:\n");
                        for booster in &selected_human_character.get_boosters() {
                            output.push_str(
                                &format!("\t{}\n", booster)[..]
                            );
                        }
                    } else {
                        output.push_str(
                            &format!("You lost {}-{} ({}).\n", self.human.points, self.computer.points, get_victory_term_by_margin(self.computer.points - self.human.points))[..]
                        );

                        return command_line_app::Prompt {
                            text: output,
                            is_final: true,
                        };
                    }
                }
            } else {
                self.computer.points += self.human.penalize_waits(3);
                output = format!("\"{}\" is not a character. 3 wait penalty!\nThe score is now {}-{}.\n\n", response, self.human.points, self.computer.points);

                if self.computer.points  < 5 {
                    output.push_str("Choose a character:\n\tNinja\n\tZombie\n\tSamurai\n\tClown\n")
                } else {
                    output.push_str(
                        &format!("You lost {}-{} ({}).\n", self.human.points, self.computer.points, get_victory_term_by_margin(self.computer.points - self.human.points))[..]
                    );

                    return command_line_app::Prompt {
                        text: output,
                        is_final: true,
                    };
                }
            }
        }

        command_line_app::Prompt {
            text: output,
            is_final: false
        }
    }
}
