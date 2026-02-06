//! Block Puzzle Solver Library
//!
//! Provides the core puzzle-solving functionality for cube packing puzzles.

pub mod geometry;
pub mod grid;
pub mod persistence;
pub mod pieces;
mod solver;

use pieces::{PlacedPiece, Puzzle};

/// Trait that erases compile-time puzzle parameters for dynamic dispatch.
///
/// All const generics (`DIM`, `GRID_SIZE`, `NUM_PIECES`) are hidden behind
/// the vtable, so callers can work with any puzzle without turbofish.
pub trait PuzzleOps {
    fn solve(&self, max_solutions: Option<usize>) -> Vec<Vec<PlacedPiece>>;
    fn save_solutions(&self, solutions: &[Vec<PlacedPiece>]) -> std::io::Result<()>;
    fn load_solutions(&self) -> Option<Vec<Vec<PlacedPiece>>>;
    fn count_solutions(&self) -> Option<usize>;
    fn format_solution(&self, solution: &[PlacedPiece]) -> String;
    fn num_pieces(&self) -> usize;
}

impl<const DIM: usize, const GRID_SIZE: usize, const NUM_PIECES: usize> PuzzleOps
    for Puzzle<DIM, GRID_SIZE, NUM_PIECES>
{
    fn solve(&self, max_solutions: Option<usize>) -> Vec<Vec<PlacedPiece>> {
        Puzzle::solve(self, max_solutions)
    }

    fn save_solutions(&self, solutions: &[Vec<PlacedPiece>]) -> std::io::Result<()> {
        persistence::save::<DIM, GRID_SIZE, NUM_PIECES>(solutions)
    }

    fn load_solutions(&self) -> Option<Vec<Vec<PlacedPiece>>> {
        persistence::load_all::<DIM, GRID_SIZE, NUM_PIECES>()
    }

    fn count_solutions(&self) -> Option<usize> {
        persistence::count::<DIM, GRID_SIZE, NUM_PIECES>()
    }

    fn format_solution(&self, solution: &[PlacedPiece]) -> String {
        grid::format_solution::<DIM, GRID_SIZE>(solution)
    }

    fn num_pieces(&self) -> usize {
        self.pieces.len()
    }
}
