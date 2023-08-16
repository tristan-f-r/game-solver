//! `game_solver` is a library for solving games.
//!
//! If you want to read how to properly use this library,
//! [the book](https://leodog896.github.io/game-solver/book) is
//! a great place to start.

pub mod game;
pub mod transposition;

#[cfg(feature = "rayon")]
use {dashmap::DashMap, rayon::prelude::*, std::hash::BuildHasher, std::sync::Arc};

use crate::game::{Game, ZeroSumPlayer};
use crate::transposition::{TranspositionTable, TranspositionTableScore};
use std::hash::Hash;

/// Runs the two-player minimax variant on a zero-sum game.
/// It uses alpha beta pruning (e.g. you can specify \[-1, 1\] to get only win/loss/draw moves).
///
/// This function requires a transposition table. If you only plan on running this function once,
/// you can use a the in-built `HashMap`.
fn negamax<T: Game<Player = ZeroSumPlayer> + Clone + Eq + Hash>(
    game: &T,
    transposition_table: &mut dyn TranspositionTable<T>,
    mut alpha: isize,
    mut beta: isize,
) -> isize {
    if game.is_draw() {
        return 0;
    }

    // check if this is a winning configuration
    for m in &mut game.possible_moves() {
        if game.is_winning_move(&m) {
            let mut board = game.clone();
            board.make_move(&m);
            return board.score() as isize;
        }
    }

    // fetch values from the transposition table
    {
        let score = transposition_table
            .get(game)
            .unwrap_or_else(|| TranspositionTableScore::UpperBound(game.max_score() as isize));

        match score {
            TranspositionTableScore::UpperBound(max) => {
                if beta > max {
                    beta = max;
                    if alpha >= beta {
                        return beta;
                    }
                }
            }
            TranspositionTableScore::LowerBound(min) => {
                if alpha < min {
                    alpha = min;
                    if alpha >= beta {
                        return alpha;
                    }
                }
            }
        };
    }

    // for principal variation search
    let mut first_child = true;

    for m in &mut game.possible_moves() {
        let mut board = game.clone();
        board.make_move(&m);

        let score = if first_child {
            -negamax(&board, transposition_table, -beta, -alpha)
        } else {
            let score = -negamax(&board, transposition_table, -alpha - 1, -alpha);
            if score > alpha {
                -negamax(&board, transposition_table, -beta, -alpha)
            } else {
                score
            }
        };

        // alpha-beta pruning - we can return early
        if score >= beta {
            transposition_table.insert(game.clone(), TranspositionTableScore::LowerBound(score));
            return beta;
        }

        if score > alpha {
            alpha = score;
        }

        first_child = false;
    }

    transposition_table.insert(game.clone(), TranspositionTableScore::UpperBound(alpha));

    alpha
}

/// Solves a game, returning the evaluated score.
///
/// The score of a position is defined by the best possible end result for the player whose turn it is.
/// In 2 player games, if a score > 0, then the player whose turn it is has a winning strategy.
/// If a score < 0, then the player whose turn it is has a losing strategy.
/// Else, the game is a draw (score = 0).
///
/// This uses iterative deepening.
pub fn solve<T: Game<Player = ZeroSumPlayer> + Clone + Eq + Hash>(
    game: &T,
    transposition_table: &mut dyn TranspositionTable<T>,
) -> isize {
    let mut alpha = game.min_score();
    let mut beta = game.max_score() as isize + 1;

    while alpha < beta {
        let med = alpha + (beta - alpha) / 2;
        let r = negamax(game, transposition_table, med, med + 1);

        if r <= med {
            beta = r;
        } else {
            alpha = r;
        }
    }

    alpha
}

/// Utility function to get a list of the move scores of a certain game.
/// Since its evaluating the same game, you can use the same transposition table.
///
/// If you want to evaluate the score of a board as a whole, use the `solve` function.
///
/// # Returns
///
/// An iterator of tuples of the form `(move, score)`.
pub fn move_scores<'a, T: Game<Player = ZeroSumPlayer> + Clone + Eq + Hash>(
    game: &'a T,
    transposition_table: &'a mut dyn TranspositionTable<T>,
) -> impl Iterator<Item = (T::Move, isize)> + 'a {
    game.possible_moves().map(move |m| {
        let mut board = game.clone();
        board.make_move(&m);
        // We flip the sign of the score because we want the score from the
        // perspective of the player playing the move, not the player whose turn it is.
        (m, -solve(&board, transposition_table))
    })
}

/// Parallelized version of `move_scores`. (faster by a large margin)
/// This requires the `rayon` feature to be enabled.
/// It uses rayon's parallel iterators to evaluate the scores of each move in parallel.
///
/// This also allows you to pass in your own hasher, for transposition table optimization.
///
/// # Returns
///
/// A vector of tuples of the form `(move, score)`.
#[cfg(feature = "rayon")]
pub fn par_move_scores_with_hasher<T>(
    game: &T,
    hasher: impl BuildHasher + Default + Clone + Sync + Send,
) -> Vec<(T::Move, isize)>
where
    T: Game<Player = ZeroSumPlayer> + Clone + Eq + Hash + Sync + Send,
    T::Move: Sync + Send,
{
    // we need to collect it first as we cant parallelize an already non-parallel iterator
    let all_moves = game.possible_moves().collect::<Vec<_>>();
    let hashmap = Arc::new(DashMap::with_hasher(hasher));

    all_moves
        .par_iter()
        .map(move |m| {
            let mut board = game.clone();
            board.make_move(m);
            // We flip the sign of the score because we want the score from the
            // perspective of the player playing the move, not the player whose turn it is.
            let mut map = hashmap.clone();
            ((*m).clone(), -solve(&board, &mut map))
        })
        .collect::<Vec<_>>()
}

/// Parallelized version of `move_scores`. (faster by a large margin)
/// This requires the `rayon` feature to be enabled.
/// It uses rayon's parallel iterators to evaluate the scores of each move in parallel.
///
/// By default, this uses the cryptograpphically unsecure `XxHash64` hasher.
/// If you want to use your own hasher, use [`par_move_scores_with_hasher`].
///
/// # Returns
///
/// A vector of tuples of the form `(move, score)`.
#[cfg(feature = "rayon")]
pub fn par_move_scores<T>(game: &T) -> Vec<(T::Move, isize)>
where
    T: Game<Player = ZeroSumPlayer> + Clone + Eq + Hash + Sync + Send,
    T::Move: Sync + Send,
{
    use std::{collections::hash_map::RandomState, hash::BuildHasherDefault};
    #[cfg(feature = "xxhash")]
    use twox_hash::XxHash64;

    if cfg!(feature = "xxhash") {
        par_move_scores_with_hasher(game, BuildHasherDefault::<XxHash64>::default())
    } else {
        par_move_scores_with_hasher(game, RandomState::new())
    }
}
