//! Benchmarks for `spliter`.

use criterion::{black_box, criterion_group, criterion_main, Criterion, SamplingMode};
use rayon::iter::plumbing::{bridge_unindexed, Folder, UnindexedConsumer, UnindexedProducer};
use rayon::iter::ParallelIterator;
use spliter::{ParallelSpliterator, Spliterator};

/// Enumerates the numbers that reach the given starting point when iterating
/// the [Collatz] map, by depth-first search over the [graph] of their orbits.
///
/// [Collatz]: https://en.wikipedia.org/wiki/Collatz_conjecture
/// [graph]: https://en.wikipedia.org/wiki/File:Collatz_orbits_of_the_all_integers_up_to_1000.svg
struct Collatz {
    stack: Vec<u32>,
}

impl Collatz {
    fn new(n: u32) -> Self {
        Self { stack: vec![n] }
    }
}

impl Iterator for Collatz {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.stack.pop()?;

        // n can be reached by dividing by two, as long as it doesn't overflow
        if let Some(even) = n.checked_mul(2) {
            self.stack.push(even);
        }

        // n can be reached by 3x + 1 iff (n - 1) / 3 is an odd integer
        if n > 4 && n % 6 == 4 {
            self.stack.push((n - 1) / 3);
        }

        Some(n)
    }
}

impl Spliterator for Collatz {
    fn split(&mut self) -> Option<Self> {
        let len = self.stack.len();
        if len >= 2 {
            let stack = self.stack.split_off(len / 2);
            Some(Self { stack })
        } else {
            None
        }
    }
}

/// Benchmarks for [Collatz].
fn bench_collatz(c: &mut Criterion) {
    c.benchmark_group("Collatz")
        .sample_size(10)
        .sampling_mode(SamplingMode::Flat)
        .bench_function("sequential", |b| {
            b.iter(|| Collatz::new(black_box(1)).count())
        })
        .bench_function("parallel", |b| {
            b.iter(|| Collatz::new(black_box(1)).par_split().count())
        });
}

/// Alternate implementation that increases split opportunities by buffering an
/// item during split(), allowing Rayon's plumbing to be used.
struct CollatzBuf {
    next: Option<u32>,
    stack: Vec<u32>,
}

impl CollatzBuf {
    fn new(n: u32) -> Self {
        Self {
            next: None,
            stack: vec![n],
        }
    }

    fn buffer(&mut self) {
        if self.next != None {
            return;
        }

        if let Some(n) = self.stack.pop() {
            if let Some(even) = n.checked_mul(2) {
                self.stack.push(even);
            }

            if n % 6 == 4 && n > 4 {
                self.stack.push((n - 1) / 3);
            }

            self.next = Some(n);
        }
    }

    fn par_iter(self) -> ParCollatzBuf {
        ParCollatzBuf(self)
    }
}

impl Iterator for CollatzBuf {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer();
        self.next.take()
    }
}

impl Spliterator for CollatzBuf {
    fn split(&mut self) -> Option<Self> {
        self.buffer();

        let len = self.stack.len();
        if len + (self.next.is_some() as usize) >= 2 {
            let stack = self.stack.split_off(len / 2);
            Some(Self { next: None, stack })
        } else {
            None
        }
    }
}

/// CollatzBuf wrapper that implements Rayon plumbing.
struct ParCollatzBuf(CollatzBuf);

impl UnindexedProducer for ParCollatzBuf {
    type Item = u32;

    fn split(mut self) -> (Self, Option<Self>) {
        let split = self.0.split();
        (self, split.map(Self))
    }

    fn fold_with<F>(self, folder: F) -> F
    where
        F: Folder<Self::Item>,
    {
        folder.consume_iter(self.0)
    }
}

impl ParallelIterator for ParCollatzBuf {
    type Item = u32;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge_unindexed(self, consumer)
    }
}

/// Benchmarks for [CollatzBuf].
fn bench_collatz_buf(c: &mut Criterion) {
    c.benchmark_group("CollatzBuf")
        .sample_size(10)
        .sampling_mode(SamplingMode::Flat)
        .bench_function("sequential", |b| {
            b.iter(|| CollatzBuf::new(black_box(1)).count())
        })
        .bench_function("plumbing", |b| {
            b.iter(|| CollatzBuf::new(black_box(1)).par_iter().count())
        })
        .bench_function("parallel", |b| {
            b.iter(|| CollatzBuf::new(black_box(1)).par_split().count())
        });
}

criterion_group!(benches, bench_collatz, bench_collatz_buf);
criterion_main!(benches);
