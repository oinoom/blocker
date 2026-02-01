//! Puzzle piece definitions and coordinate types.
//!
//! Each piece is defined as a set of unit cube positions in 3D space,
//! normalized to start at the origin.

/// A 3D coordinate representing a unit cube position.
pub type Coord = (i32, i32, i32);

/// Maximum number of cubes in any single piece.
pub const MAX_CUBES_PER_PIECE: usize = 4;

/// A placed piece: (original piece index, coordinates, cube count).
/// Uses fixed-size array to avoid heap allocation.
pub type PlacedPiece = (usize, [Coord; MAX_CUBES_PER_PIECE], u8);

/// Indices of the chiral mirror-image pair in `PIECES`.
pub const CHIRAL_PAIR: (usize, usize) = (4, 6);

/// The seven puzzle pieces that must fit into a 3x3x3 cube.
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
