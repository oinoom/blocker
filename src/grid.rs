//! Grid representation and operations for the 3x3x3 puzzle cube.
//!
//! The grid can be represented in two ways:
//! - `PieceGrid`: A 3D array where each cell contains a piece number (1-7) or 0 for empty
//! - `GridKey`: A flattened 27-byte array for efficient hashing and comparison

use crate::pieces::{Coord, PlacedPiece, CHIRAL_PAIR};

/// 3D grid where each cell contains a piece number (1-7) or 0 for empty.
pub type PieceGrid = [[[u8; 3]; 3]; 3];

/// Flattened 27-byte grid representation for hashing and comparison.
pub type GridKey = [u8; 27];

/// Number of cells in the 3x3x3 grid.
const GRID_SIZE: usize = 27;
/// Number of cells along one axis.
const GRID_DIM: usize = 3;

/// Number of distinct cube orientations.
const NUM_ROTATIONS: usize = 24;

/// Pre-computed lookup table for rotating grid indices.
///
/// `ROTATION_TABLE[rotation_index][source_cell]` gives the destination cell
/// index after applying the rotation around the grid center (1,1,1).
///
/// The rotation index ordering matches `geometry::ROTATIONS`.
static ROTATION_TABLE: [[u8; GRID_SIZE]; NUM_ROTATIONS] = build_rotation_table();

/// Builds the rotation lookup table at compile time.
///
/// For each of the 24 rotations and each of the 27 cells, computes where
/// that cell ends up after rotating the grid around its center point.
const fn build_rotation_table() -> [[u8; GRID_SIZE]; NUM_ROTATIONS] {
    let mut table = [[0u8; GRID_SIZE]; NUM_ROTATIONS];

    let mut rotation_index = 0;
    while rotation_index < NUM_ROTATIONS {
        let mut source_cell = 0;
        while source_cell < GRID_SIZE {
            // convert cell index to centered coordinates (-1 to 1)
            let centered_x = (source_cell / 9) as i32 - 1;
            let centered_y = ((source_cell / 3) % 3) as i32 - 1;
            let centered_z = (source_cell % 3) as i32 - 1;

            // apply the rotation (same formulas as geometry::ROTATIONS)
            let (rotated_x, rotated_y, rotated_z) = match rotation_index {
                0 => (centered_x, centered_y, centered_z),
                1 => (-centered_y, centered_x, centered_z),
                2 => (-centered_x, -centered_y, centered_z),
                3 => (centered_y, -centered_x, centered_z),
                4 => (centered_x, -centered_z, centered_y),
                5 => (centered_z, centered_x, centered_y),
                6 => (-centered_x, centered_z, centered_y),
                7 => (-centered_z, -centered_x, centered_y),
                8 => (centered_x, -centered_y, -centered_z),
                9 => (centered_y, centered_x, -centered_z),
                10 => (-centered_x, centered_y, -centered_z),
                11 => (-centered_y, -centered_x, -centered_z),
                12 => (centered_x, centered_z, -centered_y),
                13 => (-centered_z, centered_x, -centered_y),
                14 => (-centered_x, -centered_z, -centered_y),
                15 => (centered_z, -centered_x, -centered_y),
                16 => (centered_z, centered_y, -centered_x),
                17 => (-centered_y, centered_z, -centered_x),
                18 => (-centered_z, -centered_y, -centered_x),
                19 => (centered_y, -centered_z, -centered_x),
                20 => (-centered_z, centered_y, centered_x),
                21 => (-centered_y, -centered_z, centered_x),
                22 => (centered_z, -centered_y, centered_x),
                _ => (centered_y, centered_z, centered_x),
            };

            // convert back to cell index (shift from -1..1 to 0..2)
            let dest_x = (rotated_x + 1) as usize;
            let dest_y = (rotated_y + 1) as usize;
            let dest_z = (rotated_z + 1) as usize;
            let dest_cell = dest_x * 9 + dest_y * 3 + dest_z;

            table[rotation_index][source_cell] = dest_cell as u8;
            source_cell += 1;
        }
        rotation_index += 1;
    }
    table
}

/// Converts (x, y, z) coordinates to a linear cell index (0-26).
///
/// Index order is x-major, then y, then z: `idx = x*9 + y*3 + z`.
#[inline(always)]
pub const fn coord_to_idx(x: i32, y: i32, z: i32) -> usize {
    (x * 9 + y * 3 + z) as usize
}

/// Converts a linear cell index (0-26) to (x, y, z) coordinates.
///
/// This is the inverse of `coord_to_idx` and uses the same x-major ordering.
#[inline(always)]
pub const fn idx_to_coord(cell_index: usize) -> Coord {
    (
        (cell_index / 9) as i32,
        ((cell_index / 3) % 3) as i32,
        (cell_index % 3) as i32,
    )
}

/// Converts a solution (list of placed pieces) to a 3D grid.
pub fn solution_to_grid(solution: &[PlacedPiece]) -> PieceGrid {
    let mut grid = [[[0u8; 3]; 3]; 3];

    for &(piece_index, cube_positions, cube_count) in solution {
        let piece_number = (piece_index + 1) as u8;
        for &(x, y, z) in &cube_positions[..cube_count as usize] {
            grid[x as usize][y as usize][z as usize] = piece_number;
        }
    }

    grid
}

/// Flattens a 3D grid to a 1D key array.
///
/// The flattening order matches `coord_to_idx` (x-major, then y, then z).
#[inline]
pub fn grid_to_key(grid: &PieceGrid) -> GridKey {
    let mut key = [0u8; GRID_SIZE];

    for (x, yz_plane) in grid.iter().enumerate() {
        for (y, z_row) in yz_plane.iter().enumerate() {
            for (z, &piece_number) in z_row.iter().enumerate() {
                key[x * 9 + y * 3 + z] = piece_number;
            }
        }
    }

    key
}

/// Computes the canonical form of a solution under rotations and reflections.
///
/// Reflections swap the chiral pair, so the reflected key is normalized by
/// exchanging those piece IDs before comparison.
#[inline]
pub fn canonical_key(solution: &[PlacedPiece]) -> GridKey {
    let grid_key = grid_to_key(&solution_to_grid(solution));
    find_smallest_rotation_with_reflection(&grid_key)
}

/// Reflects a grid key across the x-axis (mirror through plane x=1).
#[inline]
fn reflect_key_x(original: &GridKey) -> GridKey {
    let mut reflected = [0u8; GRID_SIZE];

    for x in 0..GRID_DIM {
        for y in 0..GRID_DIM {
            for z in 0..GRID_DIM {
                let source = x * 9 + y * 3 + z;
                let dest = (GRID_DIM - 1 - x) * 9 + y * 3 + z;
                reflected[dest] = original[source];
            }
        }
    }

    reflected
}

/// Swaps the chiral pair IDs in a grid key.
#[inline]
fn swap_chiral_in_key(original: &GridKey) -> GridKey {
    let mut swapped = *original;
    let first = (CHIRAL_PAIR.0 + 1) as u8;
    let second = (CHIRAL_PAIR.1 + 1) as u8;

    for cell in &mut swapped {
        if *cell == first {
            *cell = second;
        } else if *cell == second {
            *cell = first;
        }
    }

    swapped
}

/// Finds the lexicographically smallest rotation of a grid key.
#[inline]
fn find_smallest_rotation(original: &GridKey) -> GridKey {
    let mut smallest = *original;

    // try all rotations except identity (index 0, which is the original)
    for rotation_mapping in &ROTATION_TABLE[1..] {
        let mut rotated = [0u8; GRID_SIZE];

        // apply rotation: value at source goes to destination
        for (source_cell, &dest_cell) in rotation_mapping.iter().enumerate() {
            rotated[dest_cell as usize] = original[source_cell];
        }

        if rotated < smallest {
            smallest = rotated;
        }
    }

    smallest
}

/// Finds the lexicographically smallest symmetry among rotations and reflections.
#[inline]
fn find_smallest_rotation_with_reflection(original: &GridKey) -> GridKey {
    let mut smallest = find_smallest_rotation(original);

    let reflected = reflect_key_x(original);
    let reflected = swap_chiral_in_key(&reflected);
    let reflected_smallest = find_smallest_rotation(&reflected);

    if reflected_smallest < smallest {
        smallest = reflected_smallest;
    }

    smallest
}

/// Formats a solution as a human-readable string.
///
/// Displays three z-slices side by side, with piece numbers 1-7.
/// Empty cells (which shouldn't exist in a complete solution) show as '.'.
///
/// Layout details:
/// - Each output row corresponds to a fixed `y` level (printed from 2 down to 0).
/// - Each row prints `z=0`, `z=1`, `z=2` slices left to right.
/// - Within a slice, `x` increases left to right.
pub fn format_solution(solution: &[PlacedPiece]) -> String {
    let grid = solution_to_grid(solution);

    let mut output = String::from("  z=0     z=1     z=2\n");

    // Print rows from top (y=2) to bottom (y=0)
    for y in (0..3).rev() {
        // Print all three z-slices for this y row
        for z in 0..3 {
            for x_plane in &grid {
                let piece_number = x_plane[y][z];
                let display_char = if piece_number == 0 {
                    '.'
                } else {
                    char::from(b'0' + piece_number)
                };
                output.push(display_char);
            }
            output.push_str("  ");
        }
        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_rotation_is_unchanged() {
        for cell in 0..GRID_SIZE {
            assert_eq!(
                ROTATION_TABLE[0][cell], cell as u8,
                "Identity rotation should not move cell {cell}"
            );
        }
    }

    #[test]
    fn test_coordinate_conversion_roundtrip() {
        for original_index in 0..GRID_SIZE {
            let (x, y, z) = idx_to_coord(original_index);
            let recovered_index = coord_to_idx(x, y, z);
            assert_eq!(
                recovered_index, original_index,
                "Roundtrip failed for index {original_index}"
            );
        }
    }
}
