use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use cgolrs::{GameOfLife, Pos2};

fn make_alive(width: i32, height: i32) -> Vec<Pos2> {
    let mut alive = Vec::new();
    for y in 0..height {
        for x in 0..width {
            if (x + y) % 3 == 0 {
                alive.push(Pos2 { x, y });
            }
        }
    }
    alive.sort();
    alive
}

fn bench_next_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("next_generation");
    for size in [64, 128, 256] {
        let alive = make_alive(size, size);

        group.bench_with_input(BenchmarkId::new("serial", size), &alive, |b, alive| {
            b.iter_batched(
                || GameOfLife::from_alive(alive.clone()),
                |mut game| game.next_generation(),
                BatchSize::LargeInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("parallel", size), &alive, |b, alive| {
            b.iter_batched(
                || GameOfLife::from_alive(alive.clone()),
                |mut game| game.next_generation_parallel(),
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, bench_next_generation);
criterion_main!(benches);
