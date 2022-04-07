mod util;

use util::cube::PocketCube;
use util::dfs::DepthFirstSearch;

use rayon::iter::plumbing::{bridge_unindexed, Folder, UnindexedConsumer, UnindexedProducer};
use rayon::iter::ParallelIterator;

impl UnindexedProducer for DepthFirstSearch {
    type Item = <Self as Iterator>::Item;

    fn split(mut self) -> (Self, Option<Self>) {
        let split = self.try_split();
        (self, split)
    }

    fn fold_with<F>(self, folder: F) -> F
    where
        F: Folder<Self::Item>,
    {
        folder.consume_iter(self)
    }
}

impl ParallelIterator for DepthFirstSearch {
    type Item = <Self as UnindexedProducer>::Item;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge_unindexed(self, consumer)
    }
}

fn main() {
    let impossible = PocketCube::impossible();
    let cubes = DepthFirstSearch::new(PocketCube::solved());
    assert!(cubes.all(|cube| cube != impossible));
}
