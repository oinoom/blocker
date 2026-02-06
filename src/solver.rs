//! Optimized backtracking puzzle solver.
//!
//! Key optimizations:
//! - Bitmask for occupied cells (u32 or u64 selected by grid size)
//! - Pre-computed orientation bitmasks for instant collision detection
//! - FxHashSet for faster state deduplication
//! - Fixed-size arrays to avoid heap allocations in hot loop
//! - Bitmask for remaining pieces (u32 for up to 32 pieces)

use rustc_hash::FxHashSet;

use crate::geometry::all_orientations;
use crate::grid::{coord_to_idx, idx_to_coord};
use crate::pieces::{Coord, PlacedPiece, Puzzle, MAX_CUBES};

/// A piece orientation: the cube positions after rotation and normalization.
type Orientation = Vec<Coord>;

/// Trait for bitmask types used to track occupied grid cells.
///
/// Implemented for `u32` (grids up to 32 cells) and `u64` (up to 64 cells).
trait CellMask: Copy + Eq + std::ops::BitAnd<Output = Self> + std::ops::BitOr<Output = Self> {
    fn zero() -> Self;
    fn all_filled(grid_size: usize) -> Self;
    fn bit(index: usize) -> Self;
    fn trailing_ones(self) -> usize;
    fn is_nonzero(self) -> bool;
}

impl CellMask for u32 {
    #[inline(always)]
    fn zero() -> Self { 0 }
    #[inline(always)]
    fn all_filled(grid_size: usize) -> Self {
        if grid_size == 32 { u32::MAX } else { (1u32 << grid_size) - 1 }
    }
    #[inline(always)]
    fn bit(index: usize) -> Self { 1u32 << index }
    #[inline(always)]
    fn trailing_ones(self) -> usize { self.trailing_ones() as usize }
    #[inline(always)]
    fn is_nonzero(self) -> bool { self != 0 }
}

impl CellMask for u64 {
    #[inline(always)]
    fn zero() -> Self { 0 }
    #[inline(always)]
    fn all_filled(grid_size: usize) -> Self {
        if grid_size == 64 { u64::MAX } else { (1u64 << grid_size) - 1 }
    }
    #[inline(always)]
    fn bit(index: usize) -> Self { 1u64 << index }
    #[inline(always)]
    fn trailing_ones(self) -> usize { self.trailing_ones() as usize }
    #[inline(always)]
    fn is_nonzero(self) -> bool { self != 0 }
}

/// Pre-computed placement data for a piece orientation at a specific position.
#[derive(Clone, Copy)]
struct Placement<M: CellMask> {
    // bitmask used for fast overlap checks
    occupied_mask: M,
    // absolute cube positions used to build output solutions
    cube_positions: [Coord; MAX_CUBES],
    // number of valid coordinates in cube_positions
    cube_count: u8,
}

/// A partial solution in the iterative backtracking search.
#[derive(Clone, Copy)]
struct PartialSolution<const NUM_PIECES: usize, M: CellMask> {
    // placed pieces in this search path
    placed_pieces: [PlacedPiece; NUM_PIECES],
    // number of valid entries in placed_pieces
    placed_count: usize,
    // bit i set means piece i is still available
    remaining_pieces: u32,
    // bit i set means grid cell i is occupied
    occupied_cells: M,
    // next piece index to scan in this frame
    current_piece_index: usize,
    // next placement index for current piece at target cell
    current_orientation_index: usize,
}

// lookup by piece then target cell then valid placements for that target
type PlacementTable<M> = Vec<Vec<Vec<Placement<M>>>>;

impl<const DIM: usize, const GRID_SIZE: usize, const NUM_PIECES: usize>
    Puzzle<DIM, GRID_SIZE, NUM_PIECES>
{
    /// Finds unique solutions, up to an optional limit.
    ///
    /// Automatically selects `u32` bitmasks for grids up to 32 cells and `u64`
    /// for larger grids.
    pub fn solve(&self, max_solutions: Option<usize>) -> Vec<Vec<PlacedPiece>> {
        if GRID_SIZE <= 32 {
            self.solve_with_mask::<u32>(max_solutions)
        } else {
            self.solve_with_mask::<u64>(max_solutions)
        }
    }

    fn solve_with_mask<M: CellMask>(
        &self,
        max_solutions: Option<usize>,
    ) -> Vec<Vec<PlacedPiece>> {
        let placement_table = Self::build_placement_table(self.pieces);
        let num_pieces = self.pieces.len();

        let mut solutions = Vec::new();
        let mut seen_states: FxHashSet<[u8; GRID_SIZE]> = FxHashSet::default();

        let initial_remaining = if num_pieces == 32 {
            // avoid shifting by 32 on u32
            u32::MAX
        } else {
            (1u32 << num_pieces) - 1
        };
        let empty_piece = PlacedPiece::EMPTY;

        // explicit dfs stack so we can resume parent states without recursion
        let mut search_stack = vec![PartialSolution {
            placed_pieces: [empty_piece; NUM_PIECES],
            placed_count: 0,
            remaining_pieces: initial_remaining,
            occupied_cells: M::zero(),
            current_piece_index: 0,
            current_orientation_index: 0,
        }];

        while let Some(mut partial) = search_stack.pop() {
            // always fill the first empty cell to keep branching consistent
            let target_cell = match Self::find_first_empty_cell(partial.occupied_cells) {
                Some(cell) => cell,
                None => {
                    // no empty cell means a complete solution
                    let solution = partial.placed_pieces[..partial.placed_count].to_vec();
                    solutions.push(solution);
                    if max_solutions.is_some_and(|max| solutions.len() >= max) {
                        return solutions;
                    }
                    continue;
                }
            };

            'pieces: loop {
                // scan remaining piece bits from the current index
                let Some(piece_index) = (partial.current_piece_index..num_pieces)
                    .find(|&i| (partial.remaining_pieces & (1u32 << i)) != 0)
                else {
                    break 'pieces;
                };
                partial.current_piece_index = piece_index;

                // all placements here are precomputed to cover target_cell
                let valid_placements = &placement_table[piece_index][target_cell];

                while partial.current_orientation_index < valid_placements.len() {
                    let placement = &valid_placements[partial.current_orientation_index];
                    partial.current_orientation_index += 1;

                    // any shared bit means this placement overlaps existing cubes
                    if (partial.occupied_cells & placement.occupied_mask).is_nonzero() {
                        continue;
                    }

                    let new_occupied = partial.occupied_cells | placement.occupied_mask;
                    let new_piece = PlacedPiece {
                        piece_index,
                        positions: placement.cube_positions,
                        cube_count: placement.cube_count,
                    };

                    let mut new_placed = partial.placed_pieces;
                    new_placed[partial.placed_count] = new_piece;
                    let new_count = partial.placed_count + 1;

                    // canonical key merges equivalent states under symmetry
                    let canonical = self.canonical_key(&new_placed[..new_count]);
                    if seen_states.contains(&canonical) {
                        continue;
                    }
                    seen_states.insert(canonical);

                    // clear the bit for the piece we just placed
                    let new_remaining = partial.remaining_pieces & !(1u32 << piece_index);

                    // push parent first then child so child runs next
                    search_stack.push(partial);
                    search_stack.push(PartialSolution {
                        placed_pieces: new_placed,
                        placed_count: new_count,
                        remaining_pieces: new_remaining,
                        occupied_cells: new_occupied,
                        current_piece_index: 0,
                        current_orientation_index: 0,
                    });

                    // branch consumed one placement so descend immediately
                    break 'pieces;
                }

                // no placement worked for this piece so try next piece
                partial.current_piece_index += 1;
                partial.current_orientation_index = 0;
            }
        }

        solutions
    }

    fn build_placement_table<M: CellMask>(
        pieces: &[&[Coord]],
    ) -> PlacementTable<M> {
        let piece_orientations: Vec<Vec<Orientation>> =
            pieces.iter().map(|piece| all_orientations(piece)).collect();

        piece_orientations
            .iter()
            .map(|orientations| {
                (0..GRID_SIZE)
                    .map(|target_cell| {
                        let target_position = idx_to_coord::<DIM>(target_cell);
                        let mut placements = Vec::new();

                        for orientation in orientations {
                            // try each cube in the orientation as the anchor on target_position
                            for &anchor in orientation {
                                if let Some(placement) =
                                    Self::try_create_placement(orientation, target_position, anchor)
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

    fn try_create_placement<M: CellMask>(
        orientation: &Orientation,
        target: Coord,
        anchor: Coord,
    ) -> Option<Placement<M>> {
        let mut occupied_mask = M::zero();
        let mut cube_positions = [(0, 0, 0); MAX_CUBES];
        // shift orientation so anchor lands on target
        let offset = (
            target.0 - anchor.0,
            target.1 - anchor.1,
            target.2 - anchor.2,
        );
        let dim = DIM as i32;

        for (cube_index, &(piece_x, piece_y, piece_z)) in orientation.iter().enumerate() {
            let absolute_x = piece_x + offset.0;
            let absolute_y = piece_y + offset.1;
            let absolute_z = piece_z + offset.2;

            // reject placements that leave cube bounds
            if !(0..dim).contains(&absolute_x)
                || !(0..dim).contains(&absolute_y)
                || !(0..dim).contains(&absolute_z)
            {
                return None;
            }

            occupied_mask = occupied_mask | M::bit(coord_to_idx::<DIM>(absolute_x, absolute_y, absolute_z));
            cube_positions[cube_index] = (absolute_x, absolute_y, absolute_z);
        }

        Some(Placement {
            occupied_mask,
            cube_positions,
            cube_count: orientation.len() as u8,
        })
    }

    #[inline(always)]
    fn find_first_empty_cell<M: CellMask>(occupied: M) -> Option<usize> {
        if occupied == M::all_filled(GRID_SIZE) {
            None
        } else {
            // with filled cells as ones trailing ones reaches the first empty bit
            Some(occupied.trailing_ones())
        }
    }
}
