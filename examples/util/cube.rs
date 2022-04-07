/// The possible colors of a square on a Rubik's cube.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Color {
    White,
    Orange,
    Green,
    Red,
    Yellow,
    Blue,
}

use Color::*;

/// The six faces of a Rubik's cube.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Face {
    Up,
    Left,
    Front,
    Right,
    Down,
    Back,
}

use Face::*;

/// The possible quarter- and half-turns.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Turn {
    Clockwise(Face),
    Halfway(Face),
    CounterClockwise(Face),
}

use Turn::*;

impl Turn {
    /// Get the face being turned.
    pub fn face(self) -> Face {
        match self {
            Clockwise(face) => face,
            Halfway(face) => face,
            CounterClockwise(face) => face,
        }
    }
}

/// A 2×2×2 Rubik's cube.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PocketCube {
    /// Layout is
    ///
    ///         ┏━┯━┓
    ///         ┃0│1┃
    ///         ┠─┼─┨
    ///         ┃2│3┃
    ///     ┏━┯━╋━┿━╋━┯━┓
    ///     ┃4│5┃6│7┃8│9┃
    ///     ┠─┼─╂─┼─╂─┼─┨
    ///     ┃0│1┃2│3┃4│5┃
    ///     ┗━┷━╋━┿━╋━┷━┛
    ///         ┃6│7┃
    ///         ┠─┼─┨
    ///         ┃8│9┃
    ///         ┣━┿━┫
    ///         ┃0│1┃
    ///         ┠─┼─┨
    ///         ┃2│3┃
    ///         ┗━┷━┛
    faces: [Color; 24],
}

impl PocketCube {
    /// God's number for pocket cubes is 11 in the half turn metric.
    pub const GODS_NUMBER: u8 = 11;

    /// Create a solved cube.
    pub fn solved() -> Self {
        Self {
            faces: [
                                White,  White,
                                White,  White,
                Orange, Orange, Green,  Green,  Red, Red,
                Orange, Orange, Green,  Green,  Red, Red,
                                Yellow, Yellow,
                                Yellow, Yellow,
                                Blue,   Blue,
                                Blue,   Blue,
            ],
        }
    }

    /// Create an impossible configuration of the cube.
    pub fn impossible() -> Self {
        Self {
            faces: [
                                White,  White,
                                Orange, White,
                Orange, Green,  White,  Green,  Red, Red,
                Orange, Orange, Green,  Green,  Red, Red,
                                Yellow, Yellow,
                                Yellow, Yellow,
                                Blue,   Blue,
                                Blue,   Blue,
            ],
        }
    }

    /// Perform a single clockwise quarter-turn of a face.
    fn quarter_turn(&self, face: Face) -> Self {
        let perm = match face {
            Up => [
                         2,  0,
                         3,  1,
                 6,  7,  8,  9, 23, 22,
                10, 11, 12, 13, 14, 15,
                        16, 17,
                        18, 19,
                        20, 21,
                         5,  4,
            ],
            Down => [
                         0,  1,
                         2,  3,
                 4,  5,  6,  7,  8,  9,
                21, 20, 10, 11, 12, 13,
                        18, 16,
                        19, 17,
                        15, 14,
                        22, 23,
            ],
            Left => [
                        20,  1,
                        22,  3,
                10,  4,  0,  7,  8,  9,
                11,  5,  2, 13, 14, 15,
                         6, 17,
                        12, 19,
                        16, 21,
                        18, 23,
            ],
            Right => [
                         0,  7,
                         2, 13,
                 4,  5,  6, 17, 14,  8,
                10, 11, 12, 19, 15,  9,
                        16, 21,
                        18, 23,
                        20,  1,
                        22,  3,
            ],
            Front => [
                         0,  1,
                        11,  5,
                 4, 16, 12,  6,  2,  9,
                10, 17, 13,  7,  3, 15,
                        14,  8,
                        18, 19,
                        20, 21,
                        22, 23,
            ],
            Back => [
                         9, 15,
                         2,  3,
                 1,  5,  6,  7,  8, 19,
                 0, 11, 12, 13, 14, 18,
                        16, 17,
                         4, 10,
                        22, 20,
                        23, 21,
            ],
        };

        let mut faces = self.faces;
        for i in 0..24 {
            faces[i] = self.faces[perm[i]];
        }

        Self { faces }
    }

    /// Return a new cube with the given face turn applied.
    pub fn turn(&self, turn: Turn) -> Self {
        match turn {
            Clockwise(face) => {
                self.quarter_turn(face)
            }
            Halfway(face) => {
                self.quarter_turn(face)
                    .quarter_turn(face)
            }
            CounterClockwise(face) => {
                self.quarter_turn(face)
                    .quarter_turn(face)
                    .quarter_turn(face)
            }
        }
    }
}
