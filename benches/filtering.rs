//! Benchmarks for script filtering performance.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use nrs::filter::filter_scripts;
use nrs::package::Script;

/// Generate a vector of test scripts.
fn generate_scripts(count: usize) -> Vec<Script> {
    (0..count)
        .map(|i| {
            let name = format!("script-{:04}", i);
            let command = format!("node scripts/{}.js", name);
            if i % 5 == 0 {
                Script::with_description(&name, &command, format!("Description for {}", name))
            } else {
                Script::new(&name, &command)
            }
        })
        .collect()
}

/// Generate realistic script names.
fn generate_realistic_scripts(count: usize) -> Vec<Script> {
    let names = [
        "dev",
        "build",
        "test",
        "lint",
        "format",
        "start",
        "serve",
        "watch",
        "clean",
        "deploy",
        "build:prod",
        "build:dev",
        "test:unit",
        "test:e2e",
        "test:integration",
        "lint:fix",
        "typecheck",
        "generate",
        "migrate",
        "seed",
    ];

    (0..count)
        .map(|i| {
            let base_name = names[i % names.len()];
            let name = if i < names.len() {
                base_name.to_string()
            } else {
                format!("{}:{}", base_name, i / names.len())
            };
            let command = format!("node scripts/{}.js", name);
            Script::new(&name, &command)
        })
        .collect()
}

fn bench_filter_empty_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_empty_query");

    for size in [10, 50, 100, 500, 1000].iter() {
        let scripts = generate_scripts(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| filter_scripts(black_box(""), black_box(&scripts), false));
        });
    }

    group.finish();
}

fn bench_filter_exact_match(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_exact_match");

    for size in [10, 50, 100, 500, 1000].iter() {
        let scripts = generate_scripts(*size);
        let query = format!("script-{:04}", size / 2);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| filter_scripts(black_box(&query), black_box(&scripts), false));
        });
    }

    group.finish();
}

fn bench_filter_fuzzy_match(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_fuzzy_match");

    for size in [10, 50, 100, 500, 1000].iter() {
        let scripts = generate_scripts(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| filter_scripts(black_box("scr"), black_box(&scripts), false));
        });
    }

    group.finish();
}

fn bench_filter_no_match(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_no_match");

    for size in [10, 50, 100, 500, 1000].iter() {
        let scripts = generate_scripts(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| filter_scripts(black_box("xyz123"), black_box(&scripts), false));
        });
    }

    group.finish();
}

fn bench_filter_with_descriptions(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_with_descriptions");

    for size in [10, 50, 100, 500, 1000].iter() {
        let scripts = generate_scripts(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| filter_scripts(black_box("desc"), black_box(&scripts), true));
        });
    }

    group.finish();
}

fn bench_filter_realistic(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_realistic");

    for size in [10, 50, 100].iter() {
        let scripts = generate_realistic_scripts(*size);

        group.bench_with_input(
            BenchmarkId::new("query_dev", size),
            &("dev", &scripts),
            |b, (query, scripts)| {
                b.iter(|| filter_scripts(black_box(query), black_box(scripts), false));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("query_build", size),
            &("build", &scripts),
            |b, (query, scripts)| {
                b.iter(|| filter_scripts(black_box(query), black_box(scripts), false));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("query_t", size),
            &("t", &scripts),
            |b, (query, scripts)| {
                b.iter(|| filter_scripts(black_box(query), black_box(scripts), false));
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_filter_empty_query,
    bench_filter_exact_match,
    bench_filter_fuzzy_match,
    bench_filter_no_match,
    bench_filter_with_descriptions,
    bench_filter_realistic,
);

criterion_main!(benches);
