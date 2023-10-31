use std::sync::OnceLock;

/// The number of permutations for a 3x3 grid of cells
const PERMUTATIONS: usize = 1 << 9;
type GameOfLifeIndex = [bool; PERMUTATIONS];

/// Returns a Singleton lookup table for the Game of Life ruleset
///
/// Equivalent to calling [`generate_gol_index`] once and storing the result
pub(super) fn get_gol_index() -> &'static GameOfLifeIndex {
    static CELL: OnceLock<[bool; PERMUTATIONS]> = OnceLock::new();
    CELL.get_or_init(generate_gol_index)
}

/// Creates a lookup table for the Game of Life ruleset
///
/// The table is indexed by a 9-bit number representing a cell and its neighbors.
/// The center cell is the middle-most bit, `1 << 4`.
///
/// Returns whether the center cell should be alive or dead in its arrangement
pub(super) fn generate_gol_index() -> GameOfLifeIndex {
    const CENTER: usize = 0b000_010_000;

    let mut indices = [false; PERMUTATIONS];
    for i in 0..indices.len() {
        let neighbors = (i & !CENTER).count_ones();
        let alive = i & CENTER != 0;
        indices[i] = match (alive, neighbors) {
            (true, 2) | (_, 3) => true,
            _ => false,
        };
    }
    indices
}
