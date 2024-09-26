//! Game trait and related types.

use std::cmp::Ordering;

use crate::player::Player;

/// Represents a move outcome
#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash)]
pub enum GameState<P: Player> {
    /// It is still a player's turn - the game continues.
    Playable,
    /// The game ended in a tie - no players won
    Tie,
    // TODO: handling non-unique player wins.
    /// A player won.
    Win(P),
}

/// Defines the 'state' the game is in.
///
/// Generally used by a game solver for better optimizations.
///
/// This is usually wrapped in an Option, as there are many games that do not classify
/// as being under 'Normal' or 'Misere.' (i.e. tic-tac-toe)
#[non_exhaustive]
pub enum StateType {
    /// If a game is under 'normal play' convention, the last player to move wins.
    /// There are no ties in this variant.
    ///
    /// Learn more: <https://en.wikipedia.org/wiki/Normal_play_convention>
    Normal,
    /// If a game is under 'misere play' convention, the last player to move loses.
    /// There are no ties in this variant.
    ///
    /// Learn more: <https://en.wikipedia.org/wiki/Mis%C3%A8re#Mis%C3%A8re_game>
    Misere,
}

impl StateType {
    pub fn state<T>(&self, game: &T) -> GameState<T::Player>
    where
        T: Game,
    {
        if game.possible_moves().next().is_none() {
            GameState::Win(match self {
                Self::Misere => game.player(),
                Self::Normal => game.player().previous(),
            })
        } else {
            GameState::Playable
        }
    }
}

/// Represents a combinatorial game.
///
/// A game has three distinct variants per game:
///
/// - Game play type: Normal, Misere, Other
/// - Game partiality type: Impartial, Partizan
/// - Game player count: >0
pub trait Game: Clone {
    /// The type of move this game uses.
    type Move: Clone;

    /// The iterator type for possible moves.
    type Iter<'a>: Iterator<Item = Self::Move> + 'a
    where
        Self: 'a;

    type MoveError;

    type Player: Player;

    const STATE_TYPE: Option<StateType>;

    /// Returns the amount of moves that have been played
    fn move_count(&self) -> usize;

    /// Get the max number of moves in a game, if any.
    fn max_moves(&self) -> Option<usize>;

    /// Makes a move.
    fn make_move(&mut self, m: &Self::Move) -> Result<(), Self::MoveError>;

    /// Returns an iterator of all possible moves.
    ///
    /// If possible, this function should "guess" what the best moves are first.
    /// For example, if this is for tic tac toe, it should give the middle move first.
    /// Since "better" moves would be found first, this permits more alpha/beta cutoffs.
    fn possible_moves(&self) -> Self::Iter<'_>;

    /// Returns a reachable game in one move.
    ///
    /// Rather, this function asks if there exists some game in the possible games set
    /// which has a resolvable, positive or negative, outcome.
    /// 
    /// This function must act in the Next player's best interest.
    /// Positive games should have highest priority, then tied games, then lost games.
    /// Exact order of what game is returned doesn't matter past its outcome equivalency,
    /// as the score is dependent on move count.
    /// 
    /// (If this function returns a losing game when a positive game exists
    /// in the set of immediately resolvable games, that is a violation of this
    /// function's contract).
    /// 
    /// This function's default implementation is quite slow,
    /// and it's encouraged to use a custom implementation.
    fn find_immediately_resolvable_game(&self) -> Result<Option<Self>, Self::MoveError> {
        let mut best_non_winning_game: Option<Self> = None; 
        
        for m in &mut self.possible_moves() {
            let mut new_self = self.clone();
            new_self.make_move(&m)?;
            match new_self.state() {
                GameState::Playable => continue,
                GameState::Tie => best_non_winning_game = Some(new_self),
                GameState::Win(winning_player) => if winning_player == self.player().turn() {
                    return Ok(Some(new_self))
                } else if best_non_winning_game.is_none() {
                    best_non_winning_game = Some(new_self)
                }
            };
        }

        Ok(best_non_winning_game)
    }

    /// Returns the current state of the game.
    /// Used for verifying initialization and is commonly called.
    ///
    /// If `Self::STATE_TYPE` isn't None,
    /// the following implementation can be used:
    ///
    /// ```ignore
    /// fn state(&self) -> GameState<Self::Player> {
    ///     Self::STATE_TYPE.unwrap().state(self)
    /// }
    /// ```
    fn state(&self) -> GameState<Self::Player>;

    /// Returns the player whose turn it is.
    /// The implementation of this should be
    /// similar to either
    ///
    /// ```ignore
    /// use game_solver::game::ZeroSumPlayer;
    ///
    /// fn player(&self) -> Self::Player {
    ///     if game.move_count % 2 == 0 {
    ///        ZeroSumPlayer::One
    ///     } else {
    ///         ZeroSumPlayer::Two
    ///     }
    /// }
    /// ```
    ///
    /// or
    ///
    /// ```ignore
    /// use game_solver::game::NPlayer;
    ///
    /// fn player(&self) -> Self::Player {
    ///     NPlayer(game.move_count % game.num_players)
    /// }
    /// ```
    ///
    /// depending on the type of game.
    ///
    /// However, no implementation is provided
    /// because this does not keep track of the move count.
    fn player(&self) -> Self::Player;
}

/// Utility function to get the upper score bound of a game.
/// 
/// Essentially, score computation generally gives some max (usually max moves),
/// and penalizes the score by the amount of moves that have been made, as we're
/// trying to encourage winning in the shortest amount of time - God's algorithm.
/// 
/// Note: Despite this returning isize, this function will always be positive.
pub fn upper_bound<T: Game>(game: &T) -> isize {
    game.max_moves().map_or(isize::MAX, |m| m as isize)
}

/// Represents an outcome of a game derived by a score and a valid instance of a game.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameScoreOutcome {
    /// The inner field represents the amount of moves till a win.
    Win(usize),
    /// The inner field represents the amount of moves till a loss.
    Loss(usize),
    Tie
}

/// Utility function to convert a score to the
/// amount of moves to a win or loss, or a tie.
pub fn score_to_outcome<T: Game>(game: &T, score: isize) -> GameScoreOutcome {
    match score.cmp(&0) {
        Ordering::Greater => GameScoreOutcome::Win((-score + upper_bound(game) - game.move_count() as isize) as usize),
        Ordering::Equal => GameScoreOutcome::Tie,
        Ordering::Less => GameScoreOutcome::Loss((score + upper_bound(game) - game.move_count() as isize) as usize)
    }
}
