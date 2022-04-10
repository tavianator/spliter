//! The `spliter` crate provides a simpler way to implement Rayon's [`ParallelIterator`] trait than
//! Rayon's [`plumbing`] module.
//!
//! Implement the [`Spliterator`] trait to teach your [`Iterator`] how to split itself in half, and
//! `spliter` will wrap it into a [`ParallelIterator`] for you.  Just call [`par_split()`].
//!
//! This crate differs from Rayon's default behavior by continuing to split even after it starts
//! consuming items.  This makes it ideal for tasks like graph or tree search where the dataset can
//! grow during iteration.  See [this post] for the story behind its development.
//!
//! [`plumbing`]: rayon::iter::plumbing
//! [`par_split()`]: ParallelSpliterator#tymethod.par_split
//! [this post]: https://tavianator.com/2022/parallel_graph_search.html

#![deny(missing_docs)]

use rayon::iter::plumbing::{Folder, Reducer, UnindexedConsumer};
use rayon::iter::ParallelIterator;
use rayon::{current_num_threads, join_context};

/// An iterator that can be split.
pub trait Spliterator: Iterator + Sized {
    /// Split this iterator in two, if possible.
    fn split(&mut self) -> Option<Self>;
}

/// Converts a [Spliterator] into a [ParallelIterator].
pub trait ParallelSpliterator: Sized {
    /// Parallelize this.
    fn par_split(self) -> ParSpliter<Self>;
}

impl<T> ParallelSpliterator for T
where
    T: Spliterator + Send,
    T::Item: Send,
{
    fn par_split(self) -> ParSpliter<Self> {
        ParSpliter::new(self)
    }
}

/// An adapter from a [Spliterator] to a [ParallelIterator].
#[derive(Clone, Copy, Debug)]
pub struct ParSpliter<T> {
    /// The underlying Spliterator.
    iter: T,
    /// The number of pieces we'd like to split into.
    splits: usize,
}

impl<T: Spliterator> ParSpliter<T> {
    fn new(iter: T) -> Self {
        Self {
            iter,
            splits: current_num_threads(),
        }
    }

    fn split(&mut self) -> Option<Self> {
        if self.splits == 0 {
            return None;
        }

        if let Some(split) = self.iter.split() {
            self.splits /= 2;
            Some(Self {
                iter: split,
                splits: self.splits,
            })
        } else {
            None
        }
    }

    fn bridge<C>(&mut self, stolen: bool, consumer: C) -> C::Result
    where
        T: Send,
        C: UnindexedConsumer<T::Item>,
    {
        // Thief-splitting: start with enough splits to fill the thread pool,
        // and reset every time a job is stolen by another thread.
        if stolen {
            self.splits = current_num_threads();
        }

        let mut folder = consumer.split_off_left().into_folder();

        if self.splits == 0 {
            return folder.consume_iter(&mut self.iter).complete();
        }

        while !folder.full() {
            // Try to split
            if let Some(mut split) = self.split() {
                let (r1, r2) = (consumer.to_reducer(), consumer.to_reducer());
                let left_consumer = consumer.split_off_left();

                let (left, right) = join_context(
                    |ctx| self.bridge(ctx.migrated(), left_consumer),
                    |ctx| split.bridge(ctx.migrated(), consumer),
                );
                return r1.reduce(folder.complete(), r2.reduce(left, right));
            }

            // Otherwise, consume an item and try again
            if let Some(next) = self.iter.next() {
                folder = folder.consume(next);
            } else {
                break;
            }
        }

        folder.complete()
    }
}

impl<T> ParallelIterator for ParSpliter<T>
where
    T: Spliterator + Send,
    T::Item: Send,
{
    type Item = T::Item;

    fn drive_unindexed<C>(mut self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        self.bridge(false, consumer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_par_split() {
        struct AllNumbers {
            stack: Vec<u32>,
        }

        impl AllNumbers {
            fn new() -> Self {
                Self { stack: vec![1] }
            }
        }

        impl Iterator for AllNumbers {
            type Item = u32;

            fn next(&mut self) -> Option<Self::Item> {
                if let Some(n) = self.stack.pop() {
                    if n < 1 << 15 {
                        self.stack.push(2 * n);
                        self.stack.push(2 * n + 1);
                    }
                    Some(n)
                } else {
                    None
                }
            }
        }

        impl Spliterator for AllNumbers {
            fn split(&mut self) -> Option<Self> {
                let len = self.stack.len();
                if len >= 2 {
                    let split = self.stack.split_off(len / 2);
                    Some(Self { stack: split })
                } else {
                    None
                }
            }
        }

        assert_eq!(AllNumbers::new().count(), (1 << 16) - 1);
        assert_eq!(AllNumbers::new().par_split().count(), (1 << 16) - 1);
    }
}
