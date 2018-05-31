use super::questions::Question;
use super::notifications::Notification;

/// A `Question` and some `Notification`s.

pub struct Output {
    /// If the game is over (and therefore no user input is required), `question` will be `None`.
    pub question: Option<Question>,
    pub notifications: Vec<Notification>,
}
