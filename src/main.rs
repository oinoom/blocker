//! Block Puzzle Solver
//!
//! Solves a 3x3x3 cube packing puzzle where seven differently-shaped pieces
//! must be arranged to completely fill the cube. The solver finds all unique
//! solutions (eliminating rotational and reflection duplicates) and provides an interactive
//! 3D visualization.

mod visualization;

use clap::{Parser, Subcommand};

use blocker::{grid, persistence, pieces, solver};
use pieces::PIECES;

/// Solves a 3x3x3 cube packing puzzle and visualizes the solutions.
#[derive(Parser)]
#[command(name = "blocker")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
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

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Solve) => {
            run_solver();
        }
        Some(Command::Display) => run_display(),
        Some(Command::Count) => run_count(),
        Some(Command::ExportJs) => run_export_js(),
        None => {
            // default: solve and display
            let solutions = run_solver();
            if !solutions.is_empty() {
                println!("Controls: Left/Right navigate, Up/Down explode, R reset");
                visualization::display(solutions);
            }
        }
    }
}

/// Solves the puzzle, saves to disk, and returns the solutions.
fn run_solver() -> Vec<Vec<pieces::PlacedPiece>> {
    let solutions = solver::solve(PIECES);

    if let Err(e) = persistence::save(&solutions) {
        eprintln!("Failed to save solutions: {}", e);
    } else {
        println!("Found {} solutions", solutions.len());
        println!("Wrote solutions.txt and solutions.bin");
    }

    solutions
}

/// Loads and displays saved solutions.
fn run_display() {
    match persistence::load_all() {
        Some(solutions) => {
            println!("Loaded {} solutions", solutions.len());
            println!("Controls: Left/Right navigate, Up/Down explode, R reset");
            visualization::display(solutions);
        }
        None => {
            eprintln!("No solutions.bin found. Run 'blocker solve' first.");
        }
    }
}

/// Prints the count of saved solutions.
fn run_count() {
    match persistence::count() {
        Some(count) => println!("{} solutions", count),
        None => eprintln!("No solutions.bin found. Run 'blocker solve' first."),
    }
}

/// Exports solutions as JavaScript array for the website.
fn run_export_js() {
    let solutions = solver::solve(PIECES);

    println!("const SOLUTIONS = [");
    for (i, solution) in solutions.iter().enumerate() {
        print!("  [");
        for (j, &(piece_idx, coords, cube_count)) in solution.iter().enumerate() {
            print!("[{}, [", piece_idx);
            for (k, &(x, y, z)) in coords[..cube_count as usize].iter().enumerate() {
                print!("[{},{},{}]", x, y, z);
                if k < cube_count as usize - 1 {
                    print!(",");
                }
            }
            print!("]]");
            if j < solution.len() - 1 {
                print!(", ");
            }
        }
        print!("]");
        if i < solutions.len() - 1 {
            println!(",");
        } else {
            println!();
        }
    }
    println!("];");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solutions_snapshot() {
        let solutions = solver::solve(PIECES);

        let mut output = format!("Found {} solutions:\n\n", solutions.len());
        for (i, solution) in solutions.iter().enumerate() {
            output.push_str(&format!("Solution {}:\n", i + 1));
            output.push_str(&grid::format_solution(solution));
            output.push('\n');
        }

        insta::assert_snapshot!(output);
    }

    #[test]
    fn test_solution_count() {
        let solutions = solver::solve(PIECES);
        assert_eq!(solutions.len(), 240);
    }
}
