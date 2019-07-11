#[macro_use]
extern crate criterion;
extern crate sortedlist;

use criterion::{Criterion, ParameterizedBenchmark};
use rand::prelude::*;
use sortedlist::SortedList;
use std::collections::BTreeSet;

fn random_vec(size: u64) -> Vec<u64> {
    let mut v = (0..size).collect::<Vec<_>>();
    v.shuffle(&mut rand::thread_rng());
    v
}

fn insertions(c: &mut Criterion) {
    let sizes = vec![1_000, 10_000, 20_000, 50_000];
    c.bench(
        "range insertions",
        ParameterizedBenchmark::new(
            "insert range block size of 1000",
            |b, &input_size| {
                b.iter(|| {
                    let mut l = SortedList::new(1000);
                    for e in 0u64..input_size {
                        l.insert(e);
                    }
                    l
                })
            },
            sizes.clone(),
        )
        .with_function("insert range block size of sqrt(n)", |b, &input_size| {
            b.iter(|| {
                let mut l = SortedList::new((input_size as f64).sqrt().ceil() as usize);
                for e in 0u64..input_size {
                    l.insert(e);
                }
                l
            })
        })
        .with_function("insert range btree", |b, &input_size| {
            b.iter(|| (0u64..input_size).collect::<BTreeSet<u64>>())
        }),
    );
    c.bench(
        "shuffled insertions",
        ParameterizedBenchmark::new(
            "insert shuffled block size of 1000",
            |b, &input_size| {
                b.iter_with_setup(
                    || random_vec(input_size),
                    |v| {
                        let mut l = SortedList::new(1000);
                        for e in v {
                            l.insert(e);
                        }
                        l
                    },
                )
            },
            sizes.clone(),
        )
        .with_function("insert shuffled btree", |b, &input_size| {
            b.iter_with_setup(
                || random_vec(input_size),
                |v| v.into_iter().collect::<BTreeSet<u64>>(),
            )
        }),
    );
    c.bench(
        "iterator",
        ParameterizedBenchmark::new(
            "iterator shuffled block size of 1000",
            |b, &input_size| {
                b.iter_with_setup(
                    || {
                        let mut l = SortedList::new(1000);
                        for e in random_vec(input_size) {
                            l.insert(e);
                        }
                        l
                    },
                    |l| assert_eq!(l.iter().max(), Some(&(input_size - 1))),
                )
            },
            sizes.clone(),
        )
        .with_function("insert shuffled btree", |b, &input_size| {
            b.iter_with_setup(
                || {
                    random_vec(input_size)
                        .into_iter()
                        .collect::<BTreeSet<u64>>()
                },
                |t| assert_eq!(t.iter().max(), Some(&(input_size - 1))),
            )
        }),
    );
    c.bench(
        "contains",
        ParameterizedBenchmark::new(
            "contains shuffled block size of 1000",
            |b, &input_size| {
                b.iter_with_setup(
                    || {
                        let mut l = SortedList::new(1000);
                        for e in random_vec(input_size) {
                            l.insert(e);
                        }
                        let x = rand::random::<u64>() % input_size;
                        (l, x)
                    },
                    |(l, x)| assert!(l.contains(&x)),
                )
            },
            sizes.clone(),
        )
        .with_function("contains shuffled btree", |b, &input_size| {
            b.iter_with_setup(
                || {
                    (
                        random_vec(input_size)
                            .into_iter()
                            .collect::<BTreeSet<u64>>(),
                        rand::random::<u64>() % input_size,
                    )
                },
                |(t, x)| assert!(t.contains(&x)),
            )
        }),
    );
    c.bench(
        "remove half elements",
        ParameterizedBenchmark::new(
            "remove shuffled block size of 1000",
            |b, &input_size| {
                b.iter_with_setup(
                    || {
                        let mut l = SortedList::new(1000);
                        for e in random_vec(input_size) {
                            l.insert(e);
                        }
                        let mut to_remove = random_vec(input_size);
                        to_remove.truncate(input_size as usize / 2);
                        (l, to_remove)
                    },
                    |(mut l, v)| {
                        for x in &v {
                            l.remove(x);
                        }
                    },
                )
            },
            sizes.clone(),
        ),
    );
}

criterion_group!(benches, insertions);
criterion_main!(benches);
