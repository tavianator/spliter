mod util;

use util::cube::PocketCube;
use util::dfs::DepthFirstSearch;

use rayon::iter::ParallelIterator;
use spliter::{ParallelSpliterator, Spliterator};

impl Spliterator for DepthFirstSearch {
    fn split(&mut self) -> Option<Self> {
        self.try_split()
    }
}

fn main() {
    let impossible = PocketCube::impossible();
    let cubes = DepthFirstSearch::new(PocketCube::solved());
    assert!(cubes.par_split().all(|cube| cube != impossible));
}
