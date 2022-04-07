mod util;

use util::cube::PocketCube;
use util::dfs::DepthFirstSearch;

fn main() {
    let impossible = PocketCube::impossible();
    let mut cubes = DepthFirstSearch::new(PocketCube::solved());
    assert!(cubes.all(|cube| cube != impossible));
}
