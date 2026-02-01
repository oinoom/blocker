//! 3D visualization of puzzle solutions using kiss3d.

use kiss3d::prelude::*;

use crate::grid::solution_to_grid;
use crate::pieces::PlacedPiece;

/// Returns the display color for a given piece index (0-6).
///
/// The mapping is stable to keep colors consistent across renders.
fn piece_color(piece_index: usize) -> Color {
    match piece_index {
        0 => Color::new(1.0, 0.2, 0.2, 1.0), // red
        1 => Color::new(0.2, 1.0, 0.2, 1.0), // green
        2 => Color::new(0.2, 0.2, 1.0, 1.0), // blue
        3 => Color::new(1.0, 1.0, 0.2, 1.0), // yellow
        4 => Color::new(1.0, 0.2, 1.0, 1.0), // magenta
        5 => Color::new(0.2, 1.0, 1.0, 1.0), // cyan
        _ => Color::new(1.0, 0.6, 0.2, 1.0), // orange
    }
}

/// Represents a rendered cube in the 3D scene.
struct RenderedCube {
    /// The kiss3d scene node for this cube.
    node: SceneNode3d,
    /// The cube's position when not exploded.
    base_position: Vec3,
    /// Which piece this cube belongs to (0-6).
    piece_index: usize,
}

/// Builds the 3D scene for a solution.
///
/// Coordinate conventions:
/// - Solver coordinates use integer x, y, z in 0..=2.
/// - Rendered cubes map x->X, y->Y, z->Z in world units.
/// - The grid is centered at the origin by offsetting positions by -1.0.
///
/// Returns the rendered cubes and a map of piece centroids for explosion animation.
fn build_scene(
    scene: &mut SceneNode3d,
    solution: &[PlacedPiece],
) -> (Vec<RenderedCube>, std::collections::HashMap<usize, Vec3>) {
    /// Size of each rendered cube (slightly smaller than 1.0 for visible gaps).
    const CUBE_SIZE: f32 = 0.9;
    /// Spacing between grid cells.
    const CELL_SPACING: f32 = 1.0;
    /// Offset to center the grid around the origin.
    const CENTER_OFFSET: f32 = -1.0;

    // compute piece centroids for explosion animation
    let mut piece_centroids: std::collections::HashMap<usize, Vec3> =
        std::collections::HashMap::new();
    for &(piece_index, cube_coords, cube_count) in solution {
        let position_sum: Vec3 = cube_coords[..cube_count as usize]
            .iter()
            .map(|&(x, y, z)| Vec3::new(x as f32, y as f32, z as f32))
            .fold(Vec3::ZERO, |acc, pos| acc + pos);
        piece_centroids.insert(piece_index, position_sum / cube_count as f32);
    }

    let grid = solution_to_grid(solution);

    let mut rendered_cubes = Vec::new();
    for (x, yz_plane) in grid.iter().enumerate() {
        for (y, z_row) in yz_plane.iter().enumerate() {
            for (z, &piece_number) in z_row.iter().enumerate() {
                if piece_number > 0 {
                    let piece_index = (piece_number - 1) as usize;
                    let base_position = Vec3::new(
                        x as f32 * CELL_SPACING + CENTER_OFFSET,
                        y as f32 * CELL_SPACING + CENTER_OFFSET,
                        z as f32 * CELL_SPACING + CENTER_OFFSET,
                    );
                    let node = scene
                        .add_cube(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE)
                        .set_color(piece_color(piece_index))
                        .set_position(base_position);
                    rendered_cubes.push(RenderedCube {
                        node,
                        base_position,
                        piece_index,
                    });
                }
            }
        }
    }

    (rendered_cubes, piece_centroids)
}

/// Displays all solutions in an interactive 3D viewer.
pub fn display(solutions: Vec<Vec<PlacedPiece>>) {
    pollster::block_on(display_async(solutions));
}

async fn display_async(solutions: Vec<Vec<PlacedPiece>>) {
    if solutions.is_empty() {
        println!("No solutions to display");
        return;
    }

    let num_solutions = solutions.len();
    let mut current_solution_index = 0;

    let mut window = Window::new(&format!(
        "Solution 1/{} - [Left/Right] navigate, [Up/Down] explode, [R] reset",
        num_solutions
    ))
    .await;

    let mut camera = OrbitCamera3d::default();
    camera.set_dist(8.0);

    let mut scene = SceneNode3d::empty();
    scene
        .add_light(Light::point(100.0))
        .set_position(Vec3::new(5.0, 5.0, 5.0));

    // center point in solver coordinates for explosion direction calculation
    let grid_center = Vec3::new(1.0, 1.0, 1.0);
    let (mut rendered_cubes, mut piece_centroids) =
        build_scene(&mut scene, &solutions[current_solution_index]);

    // how much to expand pieces outward (0.0 = compact, higher = more exploded)
    let mut explosion_amount: f32 = 0.0;
    // speed at which explosion changes per keypress
    const EXPLOSION_SPEED: f32 = 0.05;
    // whether the scene needs to be rebuilt (after solution change)
    let mut needs_rebuild = false;

    loop {
        for event in window.events().iter() {
            if let kiss3d::event::WindowEvent::Key(key, action, _) = event.value {
                use kiss3d::event::{Action, Key};
                if action == Action::Press {
                    match key {
                        Key::Up => explosion_amount += EXPLOSION_SPEED,
                        Key::Down => {
                            explosion_amount = (explosion_amount - EXPLOSION_SPEED).max(0.0)
                        }
                        Key::R => explosion_amount = 0.0,
                        Key::Right => {
                            current_solution_index = (current_solution_index + 1) % num_solutions;
                            needs_rebuild = true;
                        }
                        Key::Left => {
                            current_solution_index = current_solution_index
                                .checked_sub(1)
                                .unwrap_or(num_solutions - 1);
                            needs_rebuild = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        if needs_rebuild {
            for mut cube in rendered_cubes.drain(..) {
                cube.node.remove();
            }
            let (new_cubes, new_centroids) =
                build_scene(&mut scene, &solutions[current_solution_index]);
            rendered_cubes = new_cubes;
            piece_centroids = new_centroids;
            window.set_title(&format!(
                "Solution {}/{} - [Left/Right] navigate, [Up/Down] explode, [R] reset",
                current_solution_index + 1,
                num_solutions
            ));
            needs_rebuild = false;
        }

        // update cube positions for explosion animation
        for cube in &mut rendered_cubes {
            let centroid = piece_centroids.get(&cube.piece_index).unwrap();
            let explosion_direction = (*centroid - grid_center).normalize_or_zero();
            cube.node.set_position(
                cube.base_position + explosion_direction * explosion_amount * 2.0,
            );
        }

        if !window.render_3d(&mut scene, &mut camera).await {
            break;
        }
    }
}
