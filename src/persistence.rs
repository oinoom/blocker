//! File I/O for saving and loading puzzle solutions.
//!
//! Binary format for `solutions.bin` (little endian):
//! - u32: solution count
//! - repeat per solution:
//!   - u32: piece count
//!   - repeat per piece:
//!     - u32: piece index (0-based)
//!     - u32: cube count
//!     - repeat per cube: 3 bytes (x, y, z) in the range 0..=2

use std::fs::File;
use std::io::{Read, Write};

use crate::grid::format_solution;
use crate::pieces::{PlacedPiece, MAX_CUBES_PER_PIECE};

const SOLUTIONS_BIN: &str = "solutions.bin";
const SOLUTIONS_TXT: &str = "solutions.txt";

/// Saves solutions to both binary and text files.
pub fn save(solutions: &[Vec<PlacedPiece>]) -> std::io::Result<()> {
    save_text(solutions)?;
    save_binary(solutions)?;
    Ok(())
}

/// Saves solutions in human-readable text format.
fn save_text(solutions: &[Vec<PlacedPiece>]) -> std::io::Result<()> {
    let mut file = File::create(SOLUTIONS_TXT)?;
    writeln!(file, "Found {} solutions:\n", solutions.len())?;
    for (i, solution) in solutions.iter().enumerate() {
        writeln!(file, "Solution {}:", i + 1)?;
        write!(file, "{}", format_solution(solution))?;
        writeln!(file)?;
    }
    Ok(())
}

/// Saves solutions in compact binary format for fast loading.
fn save_binary(solutions: &[Vec<PlacedPiece>]) -> std::io::Result<()> {
    let mut file = File::create(SOLUTIONS_BIN)?;

    file.write_all(&(solutions.len() as u32).to_le_bytes())?;

    for solution in solutions {
        file.write_all(&(solution.len() as u32).to_le_bytes())?;
        for &(piece_idx, coords, cube_count) in solution {
            file.write_all(&(piece_idx as u32).to_le_bytes())?;
            file.write_all(&(cube_count as u32).to_le_bytes())?;
            for &(x, y, z) in &coords[..cube_count as usize] {
                file.write_all(&[x as u8, y as u8, z as u8])?;
            }
        }
    }

    Ok(())
}

/// Loads all solutions from the binary file.
pub fn load_all() -> Option<Vec<Vec<PlacedPiece>>> {
    let mut file = File::open(SOLUTIONS_BIN).ok()?;
    let mut u32_buffer = [0u8; 4];

    file.read_exact(&mut u32_buffer).ok()?;
    let solution_count = u32::from_le_bytes(u32_buffer) as usize;

    let mut solutions = Vec::with_capacity(solution_count);

    for _ in 0..solution_count {
        file.read_exact(&mut u32_buffer).ok()?;
        let piece_count = u32::from_le_bytes(u32_buffer) as usize;

        let mut solution = Vec::with_capacity(piece_count);
        for _ in 0..piece_count {
            file.read_exact(&mut u32_buffer).ok()?;
            let piece_index = u32::from_le_bytes(u32_buffer) as usize;

            file.read_exact(&mut u32_buffer).ok()?;
            let cube_count = u32::from_le_bytes(u32_buffer) as u8;

            let mut coords = [(0, 0, 0); MAX_CUBES_PER_PIECE];
            for i in 0..cube_count as usize {
                let mut coord_buffer = [0u8; 3];
                file.read_exact(&mut coord_buffer).ok()?;
                coords[i] = (
                    coord_buffer[0] as i32,
                    coord_buffer[1] as i32,
                    coord_buffer[2] as i32,
                );
            }
            solution.push((piece_index, coords, cube_count));
        }
        solutions.push(solution);
    }

    Some(solutions)
}

/// Returns the number of saved solutions without loading them all.
pub fn count() -> Option<usize> {
    let mut file = File::open(SOLUTIONS_BIN).ok()?;
    let mut u32_buffer = [0u8; 4];
    file.read_exact(&mut u32_buffer).ok()?;
    Some(u32::from_le_bytes(u32_buffer) as usize)
}
