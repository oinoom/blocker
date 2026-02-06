//! Grid representation and operations for cube packing puzzles.
//!
//! Generic over grid dimension (`DIM`) and total cell count (`GRID_SIZE = DIM^3`).
//! The grid is represented as a flat array where each cell contains a piece
//! number (1-based) or 0 for empty.

use crate::pieces::{Coord, PlacedPiece, Puzzle};

/// Number of distinct cube orientations.
const NUM_ROTATIONS: usize = 24;

/// Builds the rotation lookup table at compile time for any grid dimension.
///
/// For each of the 24 rotations and each cell, computes where that cell ends up
/// after rotating the grid around its center point.
///
/// Uses doubled coordinates to handle both odd (3x3x3) and even (4x4x4) grids
/// without floating point: center_doubled = DIM - 1.
const fn build_rotation_table<const DIM: usize, const GRID_SIZE: usize>(
) -> [[u8; GRID_SIZE]; NUM_ROTATIONS] {
    let mut table = [[0u8; GRID_SIZE]; NUM_ROTATIONS];
    let dim_m1 = DIM as i32 - 1;

    let mut rot = 0;
    while rot < NUM_ROTATIONS {
        let mut src = 0;
        while src < GRID_SIZE {
            let x = (src / (DIM * DIM)) as i32;
            let y = ((src / DIM) % DIM) as i32;
            let z = (src % DIM) as i32;

            // doubled centered coordinates: avoids half-integer centers for even DIM
            let cx = 2 * x - dim_m1;
            let cy = 2 * y - dim_m1;
            let cz = 2 * z - dim_m1;

            // apply rotation (same formulas as geometry::ROTATIONS, on doubled coords)
            let (rx, ry, rz) = match rot {
                0 => (cx, cy, cz),
                1 => (-cy, cx, cz),
                2 => (-cx, -cy, cz),
                3 => (cy, -cx, cz),
                4 => (cx, -cz, cy),
                5 => (cz, cx, cy),
                6 => (-cx, cz, cy),
                7 => (-cz, -cx, cy),
                8 => (cx, -cy, -cz),
                9 => (cy, cx, -cz),
                10 => (-cx, cy, -cz),
                11 => (-cy, -cx, -cz),
                12 => (cx, cz, -cy),
                13 => (-cz, cx, -cy),
                14 => (-cx, -cz, -cy),
                15 => (cz, -cx, -cy),
                16 => (cz, cy, -cx),
                17 => (-cy, cz, -cx),
                18 => (-cz, -cy, -cx),
                19 => (cy, -cz, -cx),
                20 => (-cz, cy, cx),
                21 => (-cy, -cz, cx),
                22 => (cz, -cy, cx),
                _ => (cy, cz, cx),
            };

            // convert back from doubled coords to grid indices
            let dx = ((rx + dim_m1) / 2) as usize;
            let dy = ((ry + dim_m1) / 2) as usize;
            let dz = ((rz + dim_m1) / 2) as usize;
            let dest = dx * DIM * DIM + dy * DIM + dz;

            table[rot][src] = dest as u8;
            src += 1;
        }
        rot += 1;
    }
    table
}

/// Converts (x, y, z) coordinates to a linear cell index.
///
/// Index order is x-major: `idx = x * DIM * DIM + y * DIM + z`.
#[inline(always)]
pub const fn coord_to_idx<const DIM: usize>(x: i32, y: i32, z: i32) -> usize {
    (x as usize) * DIM * DIM + (y as usize) * DIM + (z as usize)
}

/// Converts a linear cell index to (x, y, z) coordinates.
#[inline(always)]
pub const fn idx_to_coord<const DIM: usize>(cell_index: usize) -> Coord {
    (
        (cell_index / (DIM * DIM)) as i32,
        ((cell_index / DIM) % DIM) as i32,
        (cell_index % DIM) as i32,
    )
}

/// Converts a solution (list of placed pieces) to a flat grid.
///
/// Each cell contains a 1-based piece number, or 0 for empty.
pub fn solution_to_grid<const DIM: usize, const GRID_SIZE: usize>(
    solution: &[PlacedPiece],
) -> [u8; GRID_SIZE] {
    let mut grid = [0u8; GRID_SIZE];

    for placed in solution {
        let piece_number = (placed.piece_index + 1) as u8;
        for &(x, y, z) in placed.cubes() {
            grid[coord_to_idx::<DIM>(x, y, z)] = piece_number;
        }
    }

    grid
}

/// Computes the canonical form of a solution under rotations and reflections.
///
/// Reflections may swap a chiral pair, so the reflected key is normalized by
/// exchanging those piece IDs before comparison when a pair is provided.
#[inline]
pub fn canonical_key<const DIM: usize, const GRID_SIZE: usize>(
    solution: &[PlacedPiece],
    chiral_pair: Option<(usize, usize)>,
) -> [u8; GRID_SIZE] {
    let grid_key = solution_to_grid::<DIM, GRID_SIZE>(solution);
    find_smallest_rotation_with_reflection::<DIM, GRID_SIZE>(&grid_key, chiral_pair)
}

/// Reflects a grid key across the x-axis (mirror through the yz center plane).
#[inline]
fn reflect_key_x<const DIM: usize, const GRID_SIZE: usize>(
    original: &[u8; GRID_SIZE],
) -> [u8; GRID_SIZE] {
    let mut reflected = [0u8; GRID_SIZE];

    for x in 0..DIM {
        for y in 0..DIM {
            for z in 0..DIM {
                let source = x * DIM * DIM + y * DIM + z;
                let dest = (DIM - 1 - x) * DIM * DIM + y * DIM + z;
                reflected[dest] = original[source];
            }
        }
    }

    reflected
}

/// Swaps the chiral pair IDs in a grid key.
#[inline]
fn swap_chiral_in_key<const GRID_SIZE: usize>(
    original: &[u8; GRID_SIZE],
    chiral_pair: (usize, usize),
) -> [u8; GRID_SIZE] {
    let mut swapped = *original;
    let first = (chiral_pair.0 + 1) as u8;
    let second = (chiral_pair.1 + 1) as u8;

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
fn find_smallest_rotation<const DIM: usize, const GRID_SIZE: usize>(
    original: &[u8; GRID_SIZE],
) -> [u8; GRID_SIZE] {
    let table: &[[u8; GRID_SIZE]; NUM_ROTATIONS] =
        &const { build_rotation_table::<DIM, GRID_SIZE>() };
    let mut smallest = *original;

    // try all rotations except identity (index 0)
    for rotation_mapping in &table[1..] {
        let mut rotated = [0u8; GRID_SIZE];

        // move each source cell value into its rotated destination
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
fn find_smallest_rotation_with_reflection<const DIM: usize, const GRID_SIZE: usize>(
    original: &[u8; GRID_SIZE],
    chiral_pair: Option<(usize, usize)>,
) -> [u8; GRID_SIZE] {
    let mut smallest = find_smallest_rotation::<DIM, GRID_SIZE>(original);

    // compare raw shape symmetries against reflected symmetries
    let mut reflected = reflect_key_x::<DIM, GRID_SIZE>(original);
    if let Some(pair) = chiral_pair {
        // normalize mirrored chiral pieces before comparing keys
        reflected = swap_chiral_in_key(&reflected, pair);
    }
    let reflected_smallest = find_smallest_rotation::<DIM, GRID_SIZE>(&reflected);

    if reflected_smallest < smallest {
        smallest = reflected_smallest;
    }

    smallest
}

/// Formats a solution as a human-readable string.
///
/// Displays DIM z-slices side by side, with piece numbers.
/// Empty cells show as '.'.
pub fn format_solution<const DIM: usize, const GRID_SIZE: usize>(
    solution: &[PlacedPiece],
) -> String {
    let grid = solution_to_grid::<DIM, GRID_SIZE>(solution);

    // header: z=0, z=1, ..., z=DIM-1
    let mut output = String::new();
    for z in 0..DIM {
        if z > 0 {
            // padding between slices: DIM chars for the slice content, plus separator
            output.push_str("  ");
        }
        output.push_str(&format!("z={:<width$}", z, width = DIM));
    }
    output.push('\n');

    // rows from top (y=DIM-1) to bottom (y=0)
    for y in (0..DIM).rev() {
        for z in 0..DIM {
            if z > 0 {
                output.push_str("  ");
            }
            for x in 0..DIM {
                let piece_number = grid[x * DIM * DIM + y * DIM + z];
                let display_char = if piece_number == 0 {
                    '.'
                } else if piece_number < 10 {
                    char::from(b'0' + piece_number)
                } else {
                    // hex for piece numbers >= 10
                    char::from(b'A' + piece_number - 10)
                };
                output.push(display_char);
            }
        }
        output.push('\n');
    }

    output
}

impl<const DIM: usize, const GRID_SIZE: usize, const NUM_PIECES: usize>
    Puzzle<DIM, GRID_SIZE, NUM_PIECES>
{
    /// Computes the canonical key for a solution, using this puzzle's chiral pair.
    pub fn canonical_key(&self, solution: &[PlacedPiece]) -> [u8; GRID_SIZE] {
        canonical_key::<DIM, GRID_SIZE>(solution, self.chiral_pair)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_rotation_is_unchanged_3x3x3() {
        let table = const { build_rotation_table::<3, 27>() };
        for cell in 0..27 {
            assert_eq!(
                table[0][cell], cell as u8,
                "Identity rotation should not move cell {cell}"
            );
        }
    }

    #[test]
    fn test_identity_rotation_is_unchanged_4x4x4() {
        let table = const { build_rotation_table::<4, 64>() };
        for cell in 0..64 {
            assert_eq!(
                table[0][cell], cell as u8,
                "Identity rotation should not move cell {cell}"
            );
        }
    }

    #[test]
    fn test_rotations_are_permutations_3x3x3() {
        let table = const { build_rotation_table::<3, 27>() };
        for rot in 0..NUM_ROTATIONS {
            let mut seen = [false; 27];
            for src in 0..27 {
                let dest = table[rot][src] as usize;
                assert!(dest < 27, "Rotation {rot} maps cell {src} to out-of-bounds {dest}");
                assert!(!seen[dest], "Rotation {rot} maps two cells to {dest}");
                seen[dest] = true;
            }
        }
    }

    #[test]
    fn test_rotations_are_permutations_4x4x4() {
        let table = const { build_rotation_table::<4, 64>() };
        for rot in 0..NUM_ROTATIONS {
            let mut seen = [false; 64];
            for src in 0..64 {
                let dest = table[rot][src] as usize;
                assert!(dest < 64, "Rotation {rot} maps cell {src} to out-of-bounds {dest}");
                assert!(!seen[dest], "Rotation {rot} maps two cells to {dest}");
                seen[dest] = true;
            }
        }
    }

    #[test]
    fn test_coordinate_conversion_roundtrip_3x3x3() {
        for idx in 0..27 {
            let (x, y, z) = idx_to_coord::<3>(idx);
            let recovered = coord_to_idx::<3>(x, y, z);
            assert_eq!(recovered, idx, "Roundtrip failed for index {idx}");
        }
    }

    #[test]
    fn test_coordinate_conversion_roundtrip_4x4x4() {
        for idx in 0..64 {
            let (x, y, z) = idx_to_coord::<4>(idx);
            assert!(
                (x as usize) < 4 && (y as usize) < 4 && (z as usize) < 4,
                "idx_to_coord::<4>({idx}) produced out-of-range ({x},{y},{z})"
            );
            let recovered = coord_to_idx::<4>(x, y, z);
            assert_eq!(recovered, idx, "Roundtrip failed for index {idx}");
        }
    }
}
