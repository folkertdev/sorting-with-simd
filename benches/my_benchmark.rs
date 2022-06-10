use criterion::{black_box, criterion_group, criterion_main, Criterion};

use rand::{distributions::Uniform, Rng}; // 0.8.0

fn random_numbers() -> Vec<i32> {
    let range = Uniform::from(i32::MIN..i32::MAX);
    rand::thread_rng()
        .sample_iter(&range)
        .take(1_000_000)
        .collect()
}

fn bench_fibs(c: &mut Criterion) {
    let input = random_numbers();

    let mut temporary = input.clone();

    gueron2015::sort(&mut input.clone());

    let mut group = c.benchmark_group("Quicksort");
    group.bench_function("gueron2015", |b| {
        b.iter(|| {
            temporary.copy_from_slice(&input);
            gueron2015::sort(black_box(&mut temporary))
        })
    });
    group.bench_function("std", |b| {
        b.iter(|| {
            temporary.copy_from_slice(&input);
            black_box(&mut temporary).sort_unstable()
        })
    });
    group.finish();
}

criterion_group!(benches, bench_fibs);
criterion_main!(benches);
