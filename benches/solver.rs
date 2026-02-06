//! Benchmarks for the block puzzle solver.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use blocker::geometry::all_orientations;
use blocker::pieces::{BEDLAM_PUZZLE, PIECES, SOMA_PUZZLE};
use blocker::PuzzleOps;

/// Benchmark the complete Soma puzzle solving process.
fn bench_solve(c: &mut Criterion) {
    c.bench_function("solve_puzzle", |b| {
        b.iter(|| black_box(&SOMA_PUZZLE).solve(None))
    });
}

/// Benchmark finding 5 Bedlam solutions.
fn bench_solve_bedlam_5(c: &mut Criterion) {
    let mut group = c.benchmark_group("bedlam");
    group.sample_size(10);
    group.bench_function("solve_5", |b| {
        b.iter(|| black_box(&BEDLAM_PUZZLE).solve(Some(5)))
    });
    group.finish();
}

/// Benchmark computing all orientations for a single piece.
fn bench_orientations(c: &mut Criterion) {
    let piece = PIECES[0];

    c.bench_function("all_orientations", |b| {
        b.iter(|| all_orientations(black_box(piece)))
    });
}

/// Benchmark computing the canonical key for a solution.
fn bench_canonical_key(c: &mut Criterion) {
    let solutions = SOMA_PUZZLE.solve(None);
    let solution = &solutions[0];

    c.bench_function("canonical_key_with_reflection", |b| {
        b.iter(|| SOMA_PUZZLE.canonical_key(black_box(solution)))
    });
}

/// Benchmark formatting a solution for display.
fn bench_format_solution(c: &mut Criterion) {
    let solutions = SOMA_PUZZLE.solve(None);
    let solution = &solutions[0];

    c.bench_function("format_solution", |b| {
        b.iter(|| SOMA_PUZZLE.format_solution(black_box(solution)))
    });
}

criterion_group!(
    benches,
    bench_solve,
    bench_solve_bedlam_5,
    bench_orientations,
    bench_canonical_key,
    bench_format_solution
);
criterion_main!(benches);
