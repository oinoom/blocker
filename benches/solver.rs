//! Benchmarks for the block puzzle solver.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use blocker::geometry::all_orientations;
use blocker::grid::{canonical_key, format_solution};
use blocker::pieces::PIECES;
use blocker::solver::solve;

/// Benchmark the complete puzzle solving process.
fn bench_solve(c: &mut Criterion) {
    c.bench_function("solve_puzzle", |b| b.iter(|| solve(black_box(PIECES))));
}

/// Benchmark computing all orientations for a single piece.
fn bench_orientations(c: &mut Criterion) {
    let piece = PIECES[0]; // L-shaped piece with 4 cubes

    c.bench_function("all_orientations", |b| {
        b.iter(|| all_orientations(black_box(piece)))
    });
}

/// Benchmark computing the canonical key for a solution.
fn bench_canonical_key(c: &mut Criterion) {
    let solutions = solve(PIECES);
    let solution = &solutions[0];

    c.bench_function("canonical_key_with_reflection", |b| {
        b.iter(|| canonical_key(black_box(solution)))
    });
}

/// Benchmark formatting a solution for display.
fn bench_format_solution(c: &mut Criterion) {
    let solutions = solve(PIECES);
    let solution = &solutions[0];

    c.bench_function("format_solution", |b| {
        b.iter(|| format_solution(black_box(solution)))
    });
}

criterion_group!(
    benches,
    bench_solve,
    bench_orientations,
    bench_canonical_key,
    bench_format_solution
);
criterion_main!(benches);
