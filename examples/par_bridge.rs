mod util;

use util::cube::PocketCube;
use util::dfs::DepthFirstSearch;

use rayon::iter::{ParallelBridge, ParallelIterator};

fn main() {
    let impossible = PocketCube::impossible();
    let cubes = DepthFirstSearch::new(PocketCube::solved());
    assert!(cubes.par_bridge().all(|cube| cube != impossible));
}
