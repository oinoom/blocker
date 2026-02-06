//! File I/O for saving and loading puzzle solutions.
//!
//! Binary format for `solutions.bin` (little endian):
//! - 4 bytes: magic (`BLKR`)
//! - u8: format version
//! - u8: puzzle dim
//! - u8: puzzle grid size
//! - u8: puzzle piece count
//! - u32: solution count
//! - repeat per solution:
//!   - u32: piece count
//!   - repeat per piece:
//!     - u32: piece index (0-based)
//!     - u32: cube count
//!     - repeat per cube: 3 bytes (x, y, z)

use std::fs::File;
use std::io::{Read, Write};

use crate::grid::format_solution;
use crate::pieces::{PlacedPiece, MAX_CUBES};

const SOLUTIONS_BIN: &str = "solutions.bin";
const SOLUTIONS_TXT: &str = "solutions.txt";
const FILE_MAGIC: [u8; 4] = *b"BLKR";
const FILE_VERSION: u8 = 1;

/// Saves solutions to both binary and text files.
pub fn save<const DIM: usize, const GRID_SIZE: usize, const NUM_PIECES: usize>(
    solutions: &[Vec<PlacedPiece>],
) -> std::io::Result<()> {
    save_text::<DIM, GRID_SIZE>(solutions)?;
    save_binary::<DIM, GRID_SIZE, NUM_PIECES>(solutions)?;
    Ok(())
}

/// Saves solutions in human-readable text format.
fn save_text<const DIM: usize, const GRID_SIZE: usize>(
    solutions: &[Vec<PlacedPiece>],
) -> std::io::Result<()> {
    let mut file = File::create(SOLUTIONS_TXT)?;
    writeln!(file, "Found {} solutions:\n", solutions.len())?;
    for (i, solution) in solutions.iter().enumerate() {
        writeln!(file, "Solution {}:", i + 1)?;
        write!(
            file,
            "{}",
            format_solution::<DIM, GRID_SIZE>(solution)
        )?;
        writeln!(file)?;
    }
    Ok(())
}

/// Saves solutions in compact binary format for fast loading.
fn save_binary<const DIM: usize, const GRID_SIZE: usize, const NUM_PIECES: usize>(
    solutions: &[Vec<PlacedPiece>],
) -> std::io::Result<()> {
    let mut file = File::create(SOLUTIONS_BIN)?;
    file.write_all(&FILE_MAGIC)?;
    file.write_all(&[FILE_VERSION, DIM as u8, GRID_SIZE as u8, NUM_PIECES as u8])?;

    file.write_all(&(solutions.len() as u32).to_le_bytes())?;

    for solution in solutions {
        file.write_all(&(solution.len() as u32).to_le_bytes())?;
        for placed in solution {
            file.write_all(&(placed.piece_index as u32).to_le_bytes())?;
            file.write_all(&(placed.cube_count as u32).to_le_bytes())?;
            for &(x, y, z) in placed.cubes() {
                file.write_all(&[x as u8, y as u8, z as u8])?;
            }
        }
    }

    Ok(())
}

#[inline]
fn read_u32<R: Read>(reader: &mut R) -> Option<u32> {
    let mut buffer = [0u8; 4];
    reader.read_exact(&mut buffer).ok()?;
    Some(u32::from_le_bytes(buffer))
}

#[inline]
fn expected_piece_mask(num_pieces: usize) -> u32 {
    if num_pieces == 32 {
        u32::MAX
    } else {
        (1u32 << num_pieces) - 1
    }
}

fn parse_solutions<const DIM: usize, const NUM_PIECES: usize>(
    file: &mut File,
    solution_count: usize,
) -> Option<Vec<Vec<PlacedPiece>>> {
    let mut solutions = Vec::with_capacity(solution_count);
    let dim = DIM as i32;
    let expected_mask = expected_piece_mask(NUM_PIECES);

    for _ in 0..solution_count {
        let piece_count = read_u32(file)? as usize;
        if piece_count != NUM_PIECES {
            return None;
        }

        let mut seen_pieces = 0u32;
        let mut solution = Vec::with_capacity(piece_count);
        for _ in 0..piece_count {
            let piece_index = read_u32(file)? as usize;
            if piece_index >= NUM_PIECES {
                return None;
            }

            let piece_bit = 1u32 << piece_index;
            if (seen_pieces & piece_bit) != 0 {
                // reject duplicated piece ids in one solution
                return None;
            }
            seen_pieces |= piece_bit;

            let cube_count = read_u32(file)? as usize;
            if cube_count == 0 || cube_count > MAX_CUBES {
                return None;
            }

            let mut positions = [(0, 0, 0); MAX_CUBES];
            for i in 0..cube_count {
                let mut coord_buffer = [0u8; 3];
                file.read_exact(&mut coord_buffer).ok()?;
                let x = coord_buffer[0] as i32;
                let y = coord_buffer[1] as i32;
                let z = coord_buffer[2] as i32;
                if x >= dim || y >= dim || z >= dim {
                    // reject out of bounds cubes for this puzzle dimension
                    return None;
                }
                positions[i] = (x, y, z);
            }

            solution.push(PlacedPiece {
                piece_index,
                positions,
                cube_count: cube_count as u8,
            });
        }

        if seen_pieces != expected_mask {
            // every piece must appear exactly once
            return None;
        }
        solutions.push(solution);
    }

    Some(solutions)
}

/// Loads all solutions from the binary file.
pub fn load_all<const DIM: usize, const GRID_SIZE: usize, const NUM_PIECES: usize>(
) -> Option<Vec<Vec<PlacedPiece>>> {
    let mut file = File::open(SOLUTIONS_BIN).ok()?;
    let mut prefix = [0u8; 4];
    file.read_exact(&mut prefix).ok()?;

    if prefix == FILE_MAGIC {
        // current format starts with magic and metadata
        let mut metadata = [0u8; 4];
        file.read_exact(&mut metadata).ok()?;
        let version = metadata[0];
        let dim = metadata[1] as usize;
        let grid_size = metadata[2] as usize;
        let piece_count = metadata[3] as usize;

        if version != FILE_VERSION
            || dim != DIM
            || grid_size != GRID_SIZE
            || piece_count != NUM_PIECES
        {
            return None;
        }

        let solution_count = read_u32(&mut file)? as usize;
        parse_solutions::<DIM, NUM_PIECES>(&mut file, solution_count)
    } else {
        // Legacy format without a header. Keep reading but validate dimensions.
        // here prefix is the old solution count field
        let solution_count = u32::from_le_bytes(prefix) as usize;
        parse_solutions::<DIM, NUM_PIECES>(&mut file, solution_count)
    }
}

/// Returns the number of saved solutions without loading them all.
pub fn count<const DIM: usize, const GRID_SIZE: usize, const NUM_PIECES: usize>() -> Option<usize> {
    let mut file = File::open(SOLUTIONS_BIN).ok()?;
    let mut prefix = [0u8; 4];
    file.read_exact(&mut prefix).ok()?;

    if prefix == FILE_MAGIC {
        let mut metadata = [0u8; 4];
        file.read_exact(&mut metadata).ok()?;
        let version = metadata[0];
        let dim = metadata[1] as usize;
        let grid_size = metadata[2] as usize;
        let piece_count = metadata[3] as usize;

        if version != FILE_VERSION
            || dim != DIM
            || grid_size != GRID_SIZE
            || piece_count != NUM_PIECES
        {
            return None;
        }

        Some(read_u32(&mut file)? as usize)
    } else {
        // Legacy format without a header. Parse to ensure compatibility.
        let solution_count = u32::from_le_bytes(prefix) as usize;
        let solutions = parse_solutions::<DIM, NUM_PIECES>(&mut file, solution_count)?;
        Some(solutions.len())
    }
}
