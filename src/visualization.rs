//! 3D visualization of puzzle solutions using kiss3d.

use kiss3d::prelude::*;

use blocker::grid::solution_to_grid;
use blocker::pieces::PlacedPiece;

/// Returns a distinct color for a piece index by spacing hues evenly.
fn piece_color(piece_index: usize, num_pieces: usize) -> Color {
    let hue = (piece_index as f32) / (num_pieces as f32);

    // HSL to RGB with saturation=0.8, lightness=0.5
    let s: f32 = 0.8;
    let l: f32 = 0.5;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = hue * 6.0;
    let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = match h_prime as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Color::new(r + m, g + m, b + m, 1.0)
}

/// Represents a rendered cube in the 3D scene.
struct RenderedCube {
    /// The kiss3d scene node for this cube.
    node: SceneNode3d,
    /// The cube's position when not exploded.
    base_position: Vec3,
    /// Which piece this cube belongs to (0-based).
    piece_index: usize,
}

/// Builds the 3D scene for a solution.
///
/// Grid is centered at the origin by offsetting positions by -(DIM-1)/2.
fn build_scene<const DIM: usize, const GRID_SIZE: usize>(
    scene: &mut SceneNode3d,
    solution: &[PlacedPiece],
    num_pieces: usize,
) -> (Vec<RenderedCube>, std::collections::HashMap<usize, Vec3>) {
    const CUBE_SIZE: f32 = 0.9;
    const CELL_SPACING: f32 = 1.0;
    let center_offset: f32 = -((DIM as f32) - 1.0) / 2.0;

    // compute piece centroids for explosion animation
    let mut piece_centroids: std::collections::HashMap<usize, Vec3> =
        std::collections::HashMap::new();
    for placed in solution {
        let position_sum: Vec3 = placed
            .cubes()
            .iter()
            .map(|&(x, y, z)| Vec3::new(x as f32, y as f32, z as f32))
            .fold(Vec3::ZERO, |acc, pos| acc + pos);
        piece_centroids.insert(placed.piece_index, position_sum / placed.cube_count as f32);
    }

    let grid = solution_to_grid::<DIM, GRID_SIZE>(solution);

    let mut rendered_cubes = Vec::new();
    for x in 0..DIM {
        for y in 0..DIM {
            for z in 0..DIM {
                let piece_number = grid[x * DIM * DIM + y * DIM + z];
                if piece_number > 0 {
                    let piece_index = (piece_number - 1) as usize;
                    let base_position = Vec3::new(
                        x as f32 * CELL_SPACING + center_offset,
                        y as f32 * CELL_SPACING + center_offset,
                        z as f32 * CELL_SPACING + center_offset,
                    );
                    let node = scene
                        .add_cube(CUBE_SIZE, CUBE_SIZE, CUBE_SIZE)
                        .set_color(piece_color(piece_index, num_pieces))
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
pub fn display<const DIM: usize, const GRID_SIZE: usize>(
    solutions: Vec<Vec<PlacedPiece>>,
    num_pieces: usize,
) {
    pollster::block_on(display_async::<DIM, GRID_SIZE>(solutions, num_pieces));
}

async fn display_async<const DIM: usize, const GRID_SIZE: usize>(
    solutions: Vec<Vec<PlacedPiece>>,
    num_pieces: usize,
) {
    if solutions.is_empty() {
        println!("No solutions to display");
        return;
    }

    let num_solutions = solutions.len();
    let mut current_solution_index = 0;

    let mut window = Window::new(&format!(
        "Solution 1/{} - [Left/Right] navigate, [W/S] explode, [R] reset",
        num_solutions
    ))
    .await;

    let mut camera = OrbitCamera3d::default();
    camera.set_dist(DIM as f32 * 2.5);

    let mut scene = SceneNode3d::empty();
    scene
        .add_light(Light::point(100.0))
        .set_position(Vec3::new(5.0, 5.0, 5.0));

    // keep center in solver coordinate space for explosion direction math
    let grid_center_val = (DIM as f32 - 1.0) / 2.0;
    let grid_center = Vec3::new(grid_center_val, grid_center_val, grid_center_val);
    let (mut rendered_cubes, mut piece_centroids) =
        build_scene::<DIM, GRID_SIZE>(&mut scene, &solutions[current_solution_index], num_pieces);

    let mut explosion_amount: f32 = 0.0;
    const EXPLOSION_SPEED: f32 = 0.05;
    let mut needs_rebuild = false;
    let mut explode_in = false;
    let mut explode_out = false;

    loop {
        for event in window.events().iter() {
            if let kiss3d::event::WindowEvent::Key(key, action, _) = event.value {
                use kiss3d::event::{Action, Key};
                let pressed = action == Action::Press;
                match key {
                    // hold keys for smooth in and out motion
                    Key::W => explode_out = pressed,
                    Key::S => explode_in = pressed,
                    Key::R if pressed => explosion_amount = 0.0,
                    Key::Right if pressed => {
                        current_solution_index = (current_solution_index + 1) % num_solutions;
                        needs_rebuild = true;
                    }
                    Key::Left if pressed => {
                        current_solution_index = current_solution_index
                            .checked_sub(1)
                            .unwrap_or(num_solutions - 1);
                        needs_rebuild = true;
                    }
                    _ => {}
                }
            }
        }

        if explode_out {
            explosion_amount += EXPLOSION_SPEED;
        }
        if explode_in {
            explosion_amount = (explosion_amount - EXPLOSION_SPEED).max(0.0);
        }

        if needs_rebuild {
            // rebuild cubes only when switching to a different solution
            for mut cube in rendered_cubes.drain(..) {
                cube.node.remove();
            }
            let (new_cubes, new_centroids) =
                build_scene::<DIM, GRID_SIZE>(&mut scene, &solutions[current_solution_index], num_pieces);
            rendered_cubes = new_cubes;
            piece_centroids = new_centroids;
            window.set_title(&format!(
                "Solution {}/{} - [Left/Right] navigate, [W/S] explode, [R] reset",
                current_solution_index + 1,
                num_solutions
            ));
            needs_rebuild = false;
        }

        for cube in &mut rendered_cubes {
            let centroid = piece_centroids.get(&cube.piece_index).unwrap();
            // move each piece away from center using its centroid direction
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
