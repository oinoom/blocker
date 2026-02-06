# Blocker

<img width="440" height="348" alt="Screenshot 2026-02-01 at 5 41 21 PM" src="https://github.com/user-attachments/assets/815c9283-1949-423e-a718-6d39c697da65" />

Blocker is a Rust solver and viewer for cube packing puzzles. It enumerates all
unique solutions (treating rotations and reflections as equivalent) and can
render them in an interactive 3D viewer.

Supported puzzles:

- **Soma cube** — 3x3x3 grid, 7 polycubes, 240 unique solutions.
- **Bedlam cube** — 4x4x4 grid, 13 polycubes.

## Features

- Backtracking solver with symmetry reduction and bitmask collision detection.
- Multiple puzzle definitions with compile-time grid sizing.
- Interactive desktop viewer powered by kiss3d.

## Requirements

- Rust toolchain (edition 2021, stable).
- An OpenGL-capable environment for the desktop viewer.

## Quick start

- Solve the Soma cube and open the viewer:
  `cargo run --release`
- Solve the Bedlam cube (first 5 solutions):
  `cargo run --release -- -p bedlam -l 5 solve`
- Display previously saved solutions:
  `cargo run --release -- display`
- Count saved solutions:
  `cargo run --release -- count`
- Export solutions as JavaScript:
  `cargo run --release -- export-js`

## Usage

```
cargo run --release -- [OPTIONS] [COMMAND]
```

### Options

| Flag | Description |
|------|-------------|
| `-p`, `--puzzle <PUZZLE>` | Which puzzle to solve: `soma` (default) or `bedlam`. |
| `-l`, `--limit <N>` | Stop after finding N solutions. |

### Commands

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
