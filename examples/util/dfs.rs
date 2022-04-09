//! Depth-first search.

use super::cube::{Face, PocketCube, Turn};

use Face::*;
use Turn::*;

/// A node in the graph of Rubik's cube states.
struct Node {
    cube: PocketCube,
    depth: u8,
    last_face: Option<Face>,
}

impl Node {
    /// Apply a face turn and return the child node.
    fn turn(&self, turn: Turn) -> Self {
        Self {
            cube: self.cube.turn(turn),
            depth: self.depth + 1,
            last_face: Some(turn.face()),
        }
    }

    /// Get the children of this node in the graph.
    fn children(&self) -> impl IntoIterator<Item = Self> + '_ {
        // Optimization 1: All pocket cubes can be solved in 11 moves or less,
        // so there's no need to search deeper than that.

        // Optimization 2: Turning the top face is equivalent to turning the
        // bottom face, just with the whole cube rotated.  Same for left/right
        // and front/back, so we only need to consider three faces, not six.

        // Optimization 3: Turning the same face twice is equivalent to turning
        // it once (or not at all).  If we keep track of the last face we turned,
        // we only need to consider turning the other two faces.

        (self.depth < PocketCube::GODS_NUMBER)
            .then(|| [Up, Right, Back])
            .into_iter()
            .flatten()
            .filter(|face| self.last_face != Some(*face))
            .flat_map(|face| [Clockwise(face), Halfway(face), CounterClockwise(face)])
            .map(|turn| self.turn(turn))
    }
}

impl From<PocketCube> for Node {
    fn from(cube: PocketCube) -> Self {
        Self {
            cube,
            depth: 0,
            last_face: None,
        }
    }
}

/// A depth-first traversal of the Rubik's cube graph.
pub struct DepthFirstSearch {
    stack: Vec<Node>,
}

impl DepthFirstSearch {
    /// Create a new search with the given starting point.
    pub fn new(cube: PocketCube) -> Self {
        Self {
            stack: vec![cube.into()],
        }
    }

    /// Split this traversal in half if possible.
    #[allow(dead_code)]
    pub fn try_split(&mut self) -> Option<Self> {
        let len = self.stack.len();
        if len >= 2 {
            // It's a stack (LIFO), so the bits at the end come before the bits at the beginning.
            // To maintain this reversed ordering we give away the bits from the beginning and
            // keep the bits from the end:
            let mut stack = self.stack.split_off(len / 2);
            std::mem::swap(&mut stack, &mut self.stack);
            Some(Self { stack })
        } else {
            None
        }
    }
}

impl Iterator for DepthFirstSearch {
    type Item = PocketCube;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            self.stack.extend(node.children());
            Some(node.cube)
        } else {
            None
        }
    }
}
