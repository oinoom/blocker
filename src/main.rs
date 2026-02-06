//! Block Puzzle Solver
//!
//! Solves cube packing puzzles where shaped pieces must be arranged to
//! completely fill a cube. Supports multiple puzzle definitions (Soma 3x3x3,
//! Bedlam 4x4x4) and provides interactive 3D visualization.

mod visualization;

use clap::{Parser, Subcommand, ValueEnum};

use blocker::{pieces, PuzzleOps};
use pieces::{PlacedPiece, Puzzle, BEDLAM_PUZZLE, SOMA_PUZZLE};

/// Solves cube packing puzzles and visualizes the solutions.
#[derive(Parser)]
#[command(name = "blocker")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Which puzzle to solve.
    #[arg(long, short, default_value = "soma")]
    puzzle: PuzzleChoice,

    /// Stop after finding this many solutions.
    #[arg(long, short)]
    limit: Option<usize>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Clone, ValueEnum)]
enum PuzzleChoice {
    Soma,
    Bedlam,
}

#[derive(Subcommand)]
enum Command {
    /// Solve the puzzle and save solutions to disk.
    Solve,
    /// Display saved solutions in an interactive 3D viewer.
    Display,
    /// Show the number of saved solutions.
    Count,
    /// Export solutions as JavaScript for the website.
    ExportJs,
}

/// Extends PuzzleOps with 3D visualization (binary-only, not in the library).
trait PuzzleDisplay: PuzzleOps {
    fn display_solutions(&self, solutions: Vec<Vec<PlacedPiece>>);
}

impl<const DIM: usize, const GRID_SIZE: usize, const NUM_PIECES: usize> PuzzleDisplay
    for Puzzle<DIM, GRID_SIZE, NUM_PIECES>
{
    fn display_solutions(&self, solutions: Vec<Vec<PlacedPiece>>) {
        visualization::display::<DIM, GRID_SIZE>(solutions, self.pieces.len());
    }
}

fn main() {
    let cli = Cli::parse();

    let puzzle: &dyn PuzzleDisplay = match cli.puzzle {
        PuzzleChoice::Soma => &SOMA_PUZZLE,
        PuzzleChoice::Bedlam => &BEDLAM_PUZZLE,
    };

    run_with_puzzle(puzzle, cli.command, cli.limit);
}

fn run_with_puzzle(
    puzzle: &dyn PuzzleDisplay,
    command: Option<Command>,
    limit: Option<usize>,
) {
    match command {
        Some(Command::Solve) => {
            run_solver(puzzle, limit);
        }
        Some(Command::Display) => run_display(puzzle),
        Some(Command::Count) => run_count(puzzle),
        Some(Command::ExportJs) => run_export_js(puzzle, limit),
        None => {
            let solutions = run_solver(puzzle, limit);
            if !solutions.is_empty() {
                println!("Controls: Left/Right navigate, W/S explode, R reset");
                puzzle.display_solutions(solutions);
            }
        }
    }
}

/// Solves the puzzle, saves to disk, and returns the solutions.
fn run_solver(puzzle: &dyn PuzzleDisplay, limit: Option<usize>) -> Vec<Vec<PlacedPiece>> {
    let solutions = puzzle.solve(limit);

    if let Err(e) = puzzle.save_solutions(&solutions) {
        eprintln!("Failed to save solutions: {}", e);
    } else {
        println!("Found {} solutions", solutions.len());
        println!("Wrote solutions.txt and solutions.bin");
    }

    solutions
}

/// Loads and displays saved solutions.
fn run_display(puzzle: &dyn PuzzleDisplay) {
    match puzzle.load_solutions() {
        Some(solutions) => {
            println!("Loaded {} solutions", solutions.len());
            println!("Controls: Left/Right navigate, W/S explode, R reset");
            puzzle.display_solutions(solutions);
        }
        None => {
            eprintln!("No compatible solutions.bin found. Run 'blocker solve' first.");
        }
    }
}

/// Prints the count of saved solutions.
fn run_count(puzzle: &dyn PuzzleDisplay) {
    match puzzle.count_solutions() {
        Some(count) => println!("{} solutions", count),
        None => eprintln!("No compatible solutions.bin found. Run 'blocker solve' first."),
    }
}

/// Exports solutions as JavaScript array for the website.
fn run_export_js(puzzle: &dyn PuzzleDisplay, limit: Option<usize>) {
    let solutions = puzzle.solve(limit);

    println!("const SOLUTIONS = [");
    for (i, solution) in solutions.iter().enumerate() {
        let pieces: Vec<String> = solution
            .iter()
            .map(|placed| {
                let cubes: Vec<String> = placed
                    .cubes()
                    .iter()
                    .map(|&(x, y, z)| format!("[{},{},{}]", x, y, z))
                    .collect();
                format!("[{}, [{}]]", placed.piece_index, cubes.join(","))
            })
            .collect();
        let trailing = if i < solutions.len() - 1 { "," } else { "" };
        println!("  [{}]{}", pieces.join(", "), trailing);
    }
    println!("];");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solutions_snapshot() {
        let solutions = SOMA_PUZZLE.solve(None);

        let mut output = format!("Found {} solutions:\n\n", solutions.len());
        for (i, solution) in solutions.iter().enumerate() {
            output.push_str(&format!("Solution {}:\n", i + 1));
            output.push_str(&SOMA_PUZZLE.format_solution(solution));
            output.push('\n');
        }

        insta::assert_snapshot!(output);
    }

    #[test]
    fn test_solution_count() {
        let solutions = SOMA_PUZZLE.solve(None);
        assert_eq!(solutions.len(), 240);
    }
}
