//! Puzzle piece definitions and coordinate types.
//!
//! Each piece is defined as a set of unit cube positions in 3D space,
//! normalized to start at the origin.

/// A 3D coordinate representing a unit cube position.
pub type Coord = (i32, i32, i32);

/// Maximum number of cubes in any single piece across all puzzles.
pub const MAX_CUBES: usize = 5;

/// Puzzle definition with compile-time parameters.
///
/// - `DIM`: grid dimension per axis (3 for Soma, 4 for Bedlam)
/// - `GRID_SIZE`: total cells in the grid (must equal DIM^3)
/// - `NUM_PIECES`: number of pieces in the puzzle
pub struct Puzzle<const DIM: usize, const GRID_SIZE: usize, const NUM_PIECES: usize> {
    /// The set of pieces in this puzzle.
    pub pieces: &'static [&'static [Coord]],
    /// Optional chiral mirror-image pair (piece indices).
    pub chiral_pair: Option<(usize, usize)>,
}

impl<const DIM: usize, const GRID_SIZE: usize, const NUM_PIECES: usize>
    Puzzle<DIM, GRID_SIZE, NUM_PIECES>
{
    /// Creates a new puzzle definition with compile-time validation.
    pub const fn new(
        pieces: &'static [&'static [Coord]],
        chiral_pair: Option<(usize, usize)>,
    ) -> Self {
        assert!(DIM * DIM * DIM == GRID_SIZE, "GRID_SIZE must equal DIM^3");
        assert!(
            pieces.len() == NUM_PIECES,
            "pieces.len() must equal NUM_PIECES"
        );
        assert!(GRID_SIZE <= 64, "GRID_SIZE must be <= 64 (u64 bitmask)");
        assert!(NUM_PIECES <= 32, "NUM_PIECES must be <= 32 (u32 bitmask)");
        let mut i = 0;
        while i < pieces.len() {
            assert!(pieces[i].len() <= MAX_CUBES, "piece exceeds MAX_CUBES");
            i += 1;
        }
        Self {
            pieces,
            chiral_pair,
        }
    }
}

/// A piece placed at specific coordinates within the grid.
///
/// Uses a fixed-size array to avoid heap allocation in the solver's hot loop.
#[derive(Clone, Copy)]
pub struct PlacedPiece {
    pub piece_index: usize,
    pub positions: [Coord; MAX_CUBES],
    pub cube_count: u8,
}

impl PlacedPiece {
    /// A zero-valued placeholder for fixed-size array initialization.
    pub const EMPTY: Self = Self {
        piece_index: 0,
        positions: [(0, 0, 0); MAX_CUBES],
        cube_count: 0,
    };

    /// Returns the valid cube positions for this piece.
    #[inline]
    pub fn cubes(&self) -> &[Coord] {
        &self.positions[..self.cube_count as usize]
    }
}

/// Indices of the chiral mirror-image pair in `PIECES`.
pub const CHIRAL_PAIR: (usize, usize) = (4, 6);

/// The seven Soma cube pieces that must fit into a 3x3x3 cube.
///
/// Each piece is defined by its constituent unit cube positions,
/// normalized so the minimum coordinates are at the origin.
pub const PIECES: &[&[Coord]] = &[
    // L-shaped piece (4 cubes)
    &[(0, 0, 0), (1, 0, 0), (2, 0, 0), (0, 1, 0)],
    // T-shaped piece (4 cubes)
    &[(0, 0, 0), (1, 0, 0), (2, 0, 0), (1, 1, 0)],
    // S-shaped piece (4 cubes)
    &[(0, 0, 0), (1, 0, 0), (1, 1, 0), (2, 1, 0)],
    // small L piece (3 cubes)
    &[(0, 0, 0), (1, 0, 0), (0, 1, 0)],
    // 3d corner piece variant A (4 cubes)
    &[(0, 0, 0), (1, 0, 0), (0, 1, 0), (1, 0, 1)],
    // 3d corner piece variant B (4 cubes)
    &[(0, 0, 0), (1, 0, 0), (0, 1, 0), (0, 0, 1)],
    // 3d corner piece variant C (4 cubes)
    &[(0, 0, 0), (1, 0, 0), (0, 1, 0), (0, 1, 1)],
];

/// Soma puzzle constants.
pub const SOMA_DIM: usize = 3;
pub const SOMA_GRID_SIZE: usize = 27;
pub const SOMA_NUM_PIECES: usize = 7;

/// Soma puzzle definition.
pub const SOMA_PUZZLE: Puzzle<SOMA_DIM, SOMA_GRID_SIZE, SOMA_NUM_PIECES> =
    Puzzle::new(PIECES, Some(CHIRAL_PAIR));

/// The thirteen Bedlam cube pieces that must fit into a 4x4x4 cube.
///
/// Coordinates are normalized so the minimum coordinates are at the origin.
pub const BEDLAM_PIECES: &[&[Coord]] = &[
    // Little Corner (4 cubes)
    &[(0, 0, 0), (0, 1, 0), (1, 0, 0), (0, 0, 1)],
    // Long Stick (5 cubes)
    &[(0, 0, 0), (1, 0, 0), (2, 0, 0), (3, 0, 0), (3, 1, 0)],
    // Hat (5 cubes)
    &[(0, 0, 0), (0, 1, 0), (1, 1, 0), (1, 2, 0), (2, 2, 0)],
    // Bucket (5 cubes)
    &[(0, 0, 0), (0, 1, 0), (1, 1, 0), (1, 2, 0), (1, 1, 1)],
    // Screw (5 cubes)
    &[(0, 0, 0), (1, 0, 0), (1, 0, 1), (1, 1, 1), (2, 1, 1)],
    // Twist (5 cubes)
    &[(0, 0, 0), (1, 0, 0), (1, 1, 0), (1, 1, 1), (2, 1, 1)],
    // Signpost (5 cubes)
    &[(0, 0, 0), (1, 0, 0), (2, 0, 0), (1, 1, 0), (1, 0, 1)],
    // Ducktail (5 cubes)
    &[(0, 0, 0), (1, 0, 0), (1, 1, 0), (2, 1, 0), (1, 0, 1)],
    // Plane (5 cubes)
    &[(0, 0, 0), (0, 1, 0), (1, 1, 0), (2, 1, 0), (1, 2, 0)],
    // Bridge (5 cubes)
    &[(0, 0, 0), (1, 0, 0), (2, 0, 0), (0, 1, 0), (2, 1, 0)],
    // Staircase (5 cubes)
    &[(0, 0, 0), (1, 0, 0), (1, 1, 0), (2, 1, 0), (2, 2, 0)],
    // Spikey Zag (5 cubes)
    &[(0, 0, 1), (0, 1, 0), (0, 1, 1), (1, 1, 0), (1, 2, 0)],
    // Middle Zig (5 cubes)
    &[(0, 0, 0), (0, 1, 0), (0, 1, 1), (1, 1, 0), (1, 2, 0)],
];

/// Bedlam puzzle constants.
pub const BEDLAM_DIM: usize = 4;
pub const BEDLAM_GRID_SIZE: usize = 64;
pub const BEDLAM_NUM_PIECES: usize = 13;

/// Bedlam puzzle definition.
pub const BEDLAM_PUZZLE: Puzzle<BEDLAM_DIM, BEDLAM_GRID_SIZE, BEDLAM_NUM_PIECES> =
    Puzzle::new(BEDLAM_PIECES, None);
