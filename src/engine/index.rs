use std::sync::OnceLock;

/// The number of permutations for a 3x3 grid of cells
const PERMUTATIONS: usize = 1 << 9;
type GameOfLifeIndex = [bool; PERMUTATIONS];

/// Returns a Singleton lookup table for the Game of Life ruleset
///
/// Equivalent to calling [`generate_gol_index`] once and storing the result
pub(super) fn get_gol_index() -> &'static GameOfLifeIndex {
    static CELL: OnceLock<GameOfLifeIndex> = OnceLock::new();
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

#[cfg(test)]
mod tests {
    use super::*;

    fn grid_value(alive: bool, neighbors: usize) -> usize {
        const CENTER: usize = 0b000_010_000;
        const NEIGHBOR_BITS: [usize; 8] = [
            0b000_000_001,
            0b000_000_010,
            0b000_000_100,
            0b000_001_000,
            0b000_100_000,
            0b001_000_000,
            0b010_000_000,
            0b100_000_000,
        ];

        let mut value = if alive { CENTER } else { 0 };
        for bit in NEIGHBOR_BITS.iter().take(neighbors) {
            value |= bit;
        }
        value
    }

    #[test]
    fn rules_match_conway_life() {
        let index = generate_gol_index();

        assert!(index[grid_value(true, 2)]);
        assert!(index[grid_value(true, 3)]);
        assert!(index[grid_value(false, 3)]);

        assert!(!index[grid_value(true, 0)]);
        assert!(!index[grid_value(true, 1)]);
        assert!(!index[grid_value(true, 4)]);
        assert!(!index[grid_value(false, 2)]);
        assert!(!index[grid_value(false, 4)]);
    }
}
