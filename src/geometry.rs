//! 3D rotation and transformation utilities.
//!
//! A cube has 24 possible orientations in 3D space (the rotation group of a cube).
//! These are the 6 ways to choose which face points up, times 4 rotations around
//! the vertical axis.

use crate::pieces::Coord;

/// All 24 rotation functions for a cube.
///
/// Organized as 6 face-up choices x 4 rotations around vertical:
/// - Rotations 0-3: +Z face up
/// - Rotations 4-7: +Y face up
/// - Rotations 8-11: -Z face up
/// - Rotations 12-15: -Y face up
/// - Rotations 16-19: +X face up
/// - Rotations 20-23: -X face up
///
/// Ordering note: the index mapping must stay in sync with the formulas in
/// `grid::build_rotation_table`, which applies the same rotations to grid cells.
pub const ROTATIONS: [fn(Coord) -> Coord; 24] = [
    // +Z face up (identity orientation), rotate around Z axis
    |(x, y, z)| (x, y, z),      // 0 degrees
    |(x, y, z)| (-y, x, z),     // 90 degrees
    |(x, y, z)| (-x, -y, z),    // 180 degrees
    |(x, y, z)| (y, -x, z),     // 270 degrees
    // +Y face up, rotate around Y axis
    |(x, y, z)| (x, -z, y),
    |(x, y, z)| (z, x, y),
    |(x, y, z)| (-x, z, y),
    |(x, y, z)| (-z, -x, y),
    // -Z face up, rotate around Z axis
    |(x, y, z)| (x, -y, -z),
    |(x, y, z)| (y, x, -z),
    |(x, y, z)| (-x, y, -z),
    |(x, y, z)| (-y, -x, -z),
    // -Y face up, rotate around Y axis
    |(x, y, z)| (x, z, -y),
    |(x, y, z)| (-z, x, -y),
    |(x, y, z)| (-x, -z, -y),
    |(x, y, z)| (z, -x, -y),
    // +X face up, rotate around X axis
    |(x, y, z)| (z, y, -x),
    |(x, y, z)| (-y, z, -x),
    |(x, y, z)| (-z, -y, -x),
    |(x, y, z)| (y, -z, -x),
    // -X face up, rotate around X axis
    |(x, y, z)| (-z, y, x),
    |(x, y, z)| (-y, -z, x),
    |(x, y, z)| (z, -y, x),
    |(x, y, z)| (y, z, x),
];

/// Generates all unique orientations of a piece.
///
/// Applies all 24 rotations to the piece, normalizes each result so that
/// the minimum coordinates are at the origin, then removes duplicates.
/// Symmetric pieces will have fewer than 24 unique orientations.
pub fn all_orientations(piece: &[Coord]) -> Vec<Vec<Coord>> {
    let mut orientations: Vec<Vec<Coord>> = ROTATIONS
        .iter()
        .map(|rotate| {
            let rotated_coords: Vec<Coord> = piece.iter().map(|&coord| rotate(coord)).collect();
            normalize_to_origin(rotated_coords)
        })
        .collect();

    // remove duplicate orientations (symmetric pieces produce duplicates)
    orientations.sort();
    orientations.dedup();
    orientations
}

/// Translates coordinates so the minimum x, y, z values are all zero.
///
/// This normalization ensures that two orientations that differ only by
/// translation will be recognized as identical.
fn normalize_to_origin(mut coords: Vec<Coord>) -> Vec<Coord> {
    let min_x = coords.iter().map(|(x, _, _)| *x).min().unwrap();
    let min_y = coords.iter().map(|(_, y, _)| *y).min().unwrap();
    let min_z = coords.iter().map(|(_, _, z)| *z).min().unwrap();

    for (x, y, z) in &mut coords {
        *x -= min_x;
        *y -= min_y;
        *z -= min_z;
    }

    coords
}
