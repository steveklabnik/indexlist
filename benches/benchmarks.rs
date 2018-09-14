#[macro_use]
extern crate criterion;

extern crate rand;

use rand::distributions::Uniform;
use rand::Rng;

extern crate indexlist;
use indexlist::IndexList;

extern crate generational_arena;
use generational_arena::Arena;

use std::collections::LinkedList;

use criterion::{Criterion, Fun};

fn criterion_benchmark(c: &mut Criterion) {
    let arena = Fun::new("arena", move |b, _| {
        let mut arena = Arena::new();

        b.iter(|| {
            arena.insert(0);
        })
    });

    let list = Fun::new("linked_list", move |b, _| {
        let mut linked_list = LinkedList::new();
        b.iter(|| {
            linked_list.push_back(0);
        })
    });

    let index_list = Fun::new("index_list", move |b, _| {
        let mut index_list = IndexList::new();

        b.iter(|| {
            index_list.push_back(0);
        })
    });

    let functions = vec![arena, list, index_list];

    // no input
    c.bench_functions("fill8", functions, 0);

    let iterations = 100_000;

    let mut list = LinkedList::new();
    let mut index_list = IndexList::new();

    let mut rng = rand::thread_rng();
    let range = Uniform::new_inclusive(0, iterations);
    let mut numbers = rng.sample_iter(&range);

    for _ in 0..iterations {
        let number = numbers.next().unwrap();
        list.push_back(number);
        index_list.push_back(number);
    }

    let needle = numbers.next().unwrap();

    let list = Fun::new("linked_list", move |b, _| {
        b.iter(|| list.iter().find(|&&n| n == needle))
    });

    let index_list = Fun::new("index_list", move |b, _| {
        b.iter(|| index_list.iter().find(|&&n| n == needle))
    });

    let functions = vec![list, index_list];

    // no input
    c.bench_functions("find_8", functions, 0);

    let list = Fun::new("linked_list", move |b, _| {
        let mut list = LinkedList::new();

        b.iter(|| {
            list.push_front(0);
        })
    });

    let index_list = Fun::new("index_list", move |b, _| {
        let mut index_list = IndexList::new();

        b.iter(|| {
            index_list.push_front(0);
        })
    });

    let functions = vec![list, index_list];

    // no input
    c.bench_functions("push_front8", functions, 0);

    let iterations = 200_000_000;

    let mut list = LinkedList::new();
    let mut index_list = IndexList::new();

    for i in 0..iterations {
        list.push_back(i);
        index_list.push_back(i);
    }

    let list = Fun::new("linked_list", move |b, _| {
        b.iter(|| {
            list.pop_front().unwrap();
        });
    });

    let index_list = Fun::new("index_list", move |b, _| {
        b.iter(|| {
            index_list.pop_front().unwrap();
        });
    });

    let functions = vec![list, index_list];

    // no input
    c.bench_functions("pop_front8", functions, 0);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
