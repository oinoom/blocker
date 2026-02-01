//! Optimized backtracking puzzle solver.
//!
//! Key optimizations:
//! - Bitmask for occupied cells (u32 instead of 3D bool array)
//! - Pre-computed orientation bitmasks for instant collision detection
//! - FxHashSet for faster state deduplication
//! - Fixed-size arrays to avoid heap allocations in hot loop
//! - Bitmask for remaining pieces instead of Vec

use rustc_hash::FxHashSet;

use crate::geometry::all_orientations;
use crate::grid::{canonical_key, coord_to_idx, idx_to_coord, GridKey};
use crate::pieces::{Coord, PlacedPiece, MAX_CUBES_PER_PIECE};

/// Number of cells in the 3x3x3 grid.
const GRID_SIZE: usize = 27;

/// A piece orientation: the cube positions after rotation and normalization.
type Orientation = Vec<Coord>;

/// All valid placements for one piece at a specific target cell.
type CellPlacements = Vec<Placement>;

/// Lookup table indexed by `[piece_index][cell_index]`.
type PlacementTable = Vec<Vec<CellPlacements>>;

/// Maximum number of pieces in the puzzle.
const MAX_PIECES: usize = 7;

/// Bitmask with all 27 cells occupied (lowest 27 bits set).
///
/// Bit 0 corresponds to cell index 0 (coord 0,0,0 via `coord_to_idx`).
const ALL_CELLS_FILLED: u32 = (1 << GRID_SIZE) - 1;

/// Pre-computed placement data for a piece orientation at a specific position.
///
/// Stores both the bitmask (for fast collision detection) and the actual
/// coordinates (for building the solution).
#[derive(Clone, Copy)]
struct Placement {
    /// Bitmask where bit `i` is set if cell `i` is occupied by this placement.
    occupied_mask: u32,
    /// Absolute coordinates of each cube in the grid.
    cube_positions: [Coord; MAX_CUBES_PER_PIECE],
    /// Number of cubes in this piece (3 or 4).
    cube_count: u8,
}

/// Builds a lookup table of all valid placements for each piece.
///
/// Only includes placements where all cubes fit within the 3x3x3 grid.
fn build_placement_table(pieces: &[&[Coord]]) -> PlacementTable {
    let piece_orientations: Vec<Vec<Orientation>> =
        pieces.iter().map(|piece| all_orientations(piece)).collect();

    piece_orientations
        .iter()
        .map(|orientations| {
            (0..GRID_SIZE)
                .map(|target_cell| {
                    let target_position = idx_to_coord(target_cell);
                    let mut placements = Vec::new();

                    for orientation in orientations {
                        for &anchor in orientation {
                            if let Some(placement) =
                                try_create_placement(orientation, target_position, anchor)
                            {
                                placements.push(placement);
                            }
                        }
                    }

                    placements
                })
                .collect()
        })
        .collect()
}

/// Attempts to create a placement by translating an orientation to a target position.
///
/// Returns `None` if any cube would fall outside the 3x3x3 grid bounds.
fn try_create_placement(
    orientation: &Orientation,
    target: Coord,
    anchor: Coord,
) -> Option<Placement> {
    let mut occupied_mask = 0u32;
    let mut cube_positions = [(0, 0, 0); MAX_CUBES_PER_PIECE];
    let offset = (
        target.0 - anchor.0,
        target.1 - anchor.1,
        target.2 - anchor.2,
    );

    for (cube_index, &(piece_x, piece_y, piece_z)) in orientation.iter().enumerate() {
        let absolute_x = piece_x + offset.0;
        let absolute_y = piece_y + offset.1;
        let absolute_z = piece_z + offset.2;

        // check if this cube is within the 3x3x3 grid bounds
        if !(0..3).contains(&absolute_x)
            || !(0..3).contains(&absolute_y)
            || !(0..3).contains(&absolute_z)
        {
            return None;
        }

        occupied_mask |= 1 << coord_to_idx(absolute_x, absolute_y, absolute_z);
        cube_positions[cube_index] = (absolute_x, absolute_y, absolute_z);
    }

    Some(Placement {
        occupied_mask,
        cube_positions,
        cube_count: orientation.len() as u8,
    })
}

/// Finds the first empty cell in the grid using the occupied bitmask.
///
/// Returns `None` if all cells are filled (puzzle complete).
#[inline(always)]
fn find_first_empty_cell(occupied: u32) -> Option<usize> {
    if occupied == ALL_CELLS_FILLED {
        None
    } else {
        // the number of trailing 1s equals the index of the first 0 bit
        // this relies on "occupied" using 1 for filled cells
        Some(occupied.trailing_ones() as usize)
    }
}

/// A partial solution in the iterative backtracking search.
///
/// Each partial solution represents a state in the search tree where we're trying
/// different pieces and orientations to fill the next empty cell.
/// Uses fixed-size arrays and bitmasks to avoid heap allocations.
#[derive(Clone, Copy)]
struct PartialSolution {
    /// Pieces placed so far in this search path.
    placed_pieces: [PlacedPiece; MAX_PIECES],
    /// Number of pieces placed so far.
    placed_count: usize,
    /// Bitmask of remaining pieces (bit i set = piece i available).
    remaining_pieces: u8,
    /// Bitmask of currently occupied cells.
    occupied_cells: u32,
    /// Current piece index (0-6) we're trying.
    current_piece_idx: usize,
    /// Index of the next orientation to try for the current piece.
    current_orientation_index: usize,
}

/// Finds all unique solutions to the puzzle.
///
/// Uses iterative backtracking with a stack to avoid recursion depth limits.
/// Solutions are deduplicated by rotations and reflections (with chiral-pair
/// swapping) during search for early pruning.
pub fn solve(pieces: &[&[Coord]]) -> Vec<Vec<PlacedPiece>> {
    let placement_table = build_placement_table(pieces);
    let num_pieces = pieces.len();

    let mut solutions = Vec::new();
    let mut seen_states: FxHashSet<GridKey> = FxHashSet::default();

    // initial partial solution with all pieces available
    let initial_remaining = (1u8 << num_pieces) - 1;
    let empty_piece: PlacedPiece = (0, [(0, 0, 0); MAX_CUBES_PER_PIECE], 0);

    let mut search_stack = vec![PartialSolution {
        placed_pieces: [empty_piece; MAX_PIECES],
        placed_count: 0,
        remaining_pieces: initial_remaining,
        occupied_cells: 0,
        current_piece_idx: 0,
        current_orientation_index: 0,
    }];

    while let Some(mut partial) = search_stack.pop() {
        // Find the first empty cell to fill
        let target_cell = match find_first_empty_cell(partial.occupied_cells) {
            Some(cell) => cell,
            None => {
                // all cells filled - found a solution
                let solution = partial.placed_pieces[..partial.placed_count].to_vec();
                solutions.push(solution);
                continue;
            }
        };

        // try each remaining piece (iterate through set bits)
        'pieces: loop {
            // find the next available piece, starting from current_piece_idx
            let Some(piece_index) = (partial.current_piece_idx..num_pieces)
                .find(|&i| (partial.remaining_pieces & (1 << i)) != 0)
            else {
                break 'pieces;
            };
            partial.current_piece_idx = piece_index;

            let valid_placements = &placement_table[piece_index][target_cell];

            // Try each orientation of this piece at the target cell
            while partial.current_orientation_index < valid_placements.len() {
                let placement = &valid_placements[partial.current_orientation_index];
                partial.current_orientation_index += 1;

                // fast collision check using bitmask AND
                if (partial.occupied_cells & placement.occupied_mask) != 0 {
                    continue;
                }

                let new_occupied = partial.occupied_cells | placement.occupied_mask;
                let new_piece: PlacedPiece =
                    (piece_index, placement.cube_positions, placement.cube_count);

                let mut new_placed = partial.placed_pieces;
                new_placed[partial.placed_count] = new_piece;
                let new_count = partial.placed_count + 1;

                // skip if we've seen this state under any rotation or reflection
                let canonical = canonical_key(&new_placed[..new_count]);
                if seen_states.contains(&canonical) {
                    continue;
                }
                seen_states.insert(canonical);

                // remove the placed piece from remaining (flip bit off)
                let new_remaining = partial.remaining_pieces & !(1 << piece_index);

                // save current partial solution for backtracking, then push new state
                search_stack.push(partial);
                search_stack.push(PartialSolution {
                    placed_pieces: new_placed,
                    placed_count: new_count,
                    remaining_pieces: new_remaining,
                    occupied_cells: new_occupied,
                    current_piece_idx: 0,
                    current_orientation_index: 0,
                });

                // placed a piece successfully; explore this branch
                break 'pieces;
            }

            // move to next piece, reset orientation counter
            partial.current_piece_idx += 1;
            partial.current_orientation_index = 0;
        }
    }

    solutions
}
