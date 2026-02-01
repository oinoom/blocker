# Blocker

<img width="440" height="348" alt="Screenshot 2026-02-01 at 5 41 21 PM" src="https://github.com/user-attachments/assets/815c9283-1949-423e-a718-6d39c697da65" />

Blocker is a Rust solver and viewer for the Soma cube, a 3x3x3 packing puzzle
made from seven polycubes. It enumerates all unique solutions (240 when
rotations and reflections are treated as equivalent) and can render them in an
interactive 3D viewer.

## Features

- Backtracking solver with symmetry reduction.
- Interactive desktop viewer powered by kiss3d.

## Requirements

- Rust toolchain (edition 2021, stable).
- An OpenGL-capable environment for the desktop viewer.

## Quick start

- Solve and display all solutions:
  `cargo run --release`
- Solve and save solutions to disk:
  `cargo run --release -- solve`
- Display previously saved solutions:
  `cargo run --release -- display`
- Count saved solutions:
  `cargo run --release -- count`
- Export solutions as JavaScript:
  `cargo run --release -- export-js`

## Commands

All commands are run via:

`cargo run --release -- <command>`

Available commands:

- `solve`      Solve the puzzle and write solutions to disk.
- `display`    Display saved solutions in the 3D viewer.
- `count`      Print the number of saved solutions.
- `export-js`  Print a JavaScript array of solutions to stdout.

If no subcommand is provided, Blocker solves the puzzle and launches the viewer.

## Outputs

`solutions.txt` and `solutions.bin` are generated in the project root when you
run the solver. You can delete them at any time; they are regenerated on the
next `solve`. The binary format is documented in `src/persistence.rs`.

## Tests and benchmarks

- Run tests:
  `cargo test`
- Run benches:
  `cargo bench`
