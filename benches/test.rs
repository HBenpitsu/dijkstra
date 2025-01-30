use criterion::*;
use dijkstra::dijkstra::*;
use dijkstra::graph::*;
use std::time::Duration;

fn sparse_instance() -> GraphNetwork<DijkstraNode, DijkstraArc> {
    let mut arcs: Vec<(usize, usize, usize)> = Vec::new();
    let number_of_nodes = 1000;
    for i in 0..number_of_nodes {
        arcs.push((i, (i + 3) % number_of_nodes, 1));
        arcs.push((i, (i + 7) % number_of_nodes, 1));
        arcs.push((i, (i + 13) % number_of_nodes, 1));
    }
    network_factory(arcs)
}

fn dense_instance() -> GraphNetwork<DijkstraNode, DijkstraArc> {
    let mut arcs: Vec<(usize, usize, usize)> = Vec::new();
    let number_of_nodes = 1000;
    for i in 0..number_of_nodes {
        for j in i..number_of_nodes {
            arcs.push((i, j, 1));
        }
    }
    network_factory(arcs)
}

fn mini_instance() -> GraphNetwork<DijkstraNode, DijkstraArc> {
    network_factory(vec![
        (0, 1, 1),
        (0, 2, 3),
        (0, 3, 2),
        (1, 2, 1),
        (3, 4, 2),
        (4, 3, 2),
        (4, 5, 2),
        (5, 3, 2),
    ])
}

fn bench_simple_dijkstra(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple");
    group.measurement_time(Duration::from_secs(30));
    group.bench_function("sparse", |b| {
        b.iter_batched(
            || sparse_instance(),
            |mut network| {
                simple_dijkstra(&mut network, 0);
            },
            BatchSize::LargeInput,
        );
    });
    group.bench_function("dense", |b| {
        b.iter_batched(
            || dense_instance(),
            |mut network| {
                simple_dijkstra(&mut network, 0);
            },
            BatchSize::LargeInput,
        );
    });
    group.bench_function("mini", |b| {
        b.iter_batched(
            || mini_instance(),
            |mut network| {
                simple_dijkstra(&mut network, 0);
            },
            BatchSize::LargeInput,
        );
    });
}

fn bench_dijkstra(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci");
    group.measurement_time(Duration::from_secs(30));
    group.bench_function("sparse", |b| {
        b.iter_batched(
            || sparse_instance(),
            |mut network| {
                dijkstra(&mut network, 0);
            },
            BatchSize::LargeInput,
        );
    });
    group.bench_function("dense", |b| {
        b.iter_batched(
            || dense_instance(),
            |mut network| {
                dijkstra(&mut network, 0);
            },
            BatchSize::LargeInput,
        );
    });
    group.bench_function("mini", |b| {
        b.iter_batched(
            || mini_instance(),
            |mut network| {
                dijkstra(&mut network, 0);
            },
            BatchSize::LargeInput,
        );
    });
}

criterion_group!(benches, bench_simple_dijkstra, bench_dijkstra);
criterion_main!(benches);
