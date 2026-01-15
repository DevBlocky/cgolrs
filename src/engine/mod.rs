mod index;
mod scan;
mod window;

use self::scan::MultiRowPosCursor;
pub use self::window::GameEngineWindow;
use crate::Pos2;
use rayon::prelude::*;
use std::ops::Range;

#[derive(Debug)]
pub struct GameOfLife {
    alive: Vec<Pos2>,
}

impl GameOfLife {
    #[inline]
    pub fn from_alive(alive: Vec<Pos2>) -> Self {
        debug_assert!(
            alive.windows(2).all(|w| w[0] < w[1]),
            "output is not properly sorted"
        );
        Self { alive }
    }

    pub fn next_generation(&mut self) {
        let next = NextGen::new(&self.alive).collect::<Vec<_>>();
        // verify integrity of next generation
        debug_assert!(
            next.windows(2).all(|w| w[0] < w[1]),
            "output is not properly sorted"
        );
        self.alive = next;
    }

    pub fn next_generation_parallel(&mut self) {
        if self.alive.is_empty() {
            return;
        }
        let next = StripedNextGen::new(&self.alive).compute();
        // verify integrity of next generation
        debug_assert!(
            next.windows(2).all(|w| w[0] < w[1]),
            "output is not properly sorted"
        );
        self.alive = next;
    }

    pub fn window(&self, top_left: Pos2, bottom_right: Pos2) -> GameEngineWindow<'_> {
        GameEngineWindow::new(self, top_left, bottom_right)
    }

    #[inline]
    pub fn alive_count(&self) -> usize {
        self.alive.len()
    }

    #[inline]
    pub fn take(self) -> Vec<Pos2> {
        self.alive
    }
}

struct NextGen<'a> {
    cursor: MultiRowPosCursor<'a>,
}
impl<'a> NextGen<'a> {
    const ROW_MASK: u8 = 0b111;
    fn next_cell_state(buffers: &[u8]) -> bool {
        // combine the first 3 bits of each bit buffer into a bit-grid
        let mut grid: usize = 0;
        for (i, &buffer) in buffers.iter().enumerate() {
            grid |= ((buffer & Self::ROW_MASK) as usize) << (i * 3);
        }

        // lookup the grid in the index to get the state of the central cell
        index::get_gol_index()[grid]
    }

    fn new(alive: &'a [Pos2]) -> Self {
        let cursor = MultiRowPosCursor::new(alive, 3);
        Self { cursor }
    }

    fn pos(&self) -> Pos2 {
        // since the returned cursor pos is the bottom most cursor, we have to adjust by one to get to the "center"
        self.cursor.cursor() - Pos2::one()
    }
    fn seek(&mut self, pos: Pos2) {
        // see above for reason to add (1, 1)
        self.cursor.seek(pos + Pos2::one());
    }
    fn step(&mut self) -> Option<(Pos2, bool)> {
        let is_empty = self
            .cursor
            .buffers()
            .iter()
            .all(|&b| b & Self::ROW_MASK == 0);

        let next_state = Self::next_cell_state(if is_empty {
            self.cursor.seek_closest()?
        } else {
            self.cursor.next()
        });
        Some((self.pos(), next_state))
    }
}
impl Iterator for NextGen<'_> {
    type Item = Pos2;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((pos, alive)) = self.step() {
            if alive {
                return Some(pos);
            }
        }
        None
    }
}

struct NextGenBand<'a> {
    inner: NextGen<'a>,
    last_pos: Pos2,
}
impl<'a> NextGenBand<'a> {
    fn new(slice: &'a [Pos2], rng: Range<usize>) -> Self {
        let first_pos = slice[rng.start] - Pos2::one(); // top left of rng.start
        let last_pos = slice[rng.end - 1] + Pos2::one(); // bottom right of rng.end

        let mut inner = NextGen::new(slice);
        inner.seek(first_pos);

        Self { inner, last_pos }
    }
}
impl Iterator for NextGenBand<'_> {
    type Item = Pos2;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(pos) = self.inner.next() {
            if pos > self.last_pos {
                return None;
            }
            return Some(pos);
        }
        None
    }
}

struct StripedNextGen<'a> {
    bands: Vec<NextGenBand<'a>>,
}
impl<'a> StripedNextGen<'a> {
    fn new(slice: &'a [Pos2]) -> Self {
        Self::with_bands(slice, rayon::current_num_threads())
    }
    fn with_bands(slice: &'a [Pos2], n: usize) -> Self {
        if slice.is_empty() || n == 0 {
            return Self { bands: Vec::new() };
        }

        let n = n.min(slice.len());
        let base = slice.len() / n;
        let remainder = slice.len() % n;
        let mut bands = Vec::with_capacity(n);
        let mut start = 0;
        for i in 0..n {
            let size = base + usize::from(i < remainder);
            let end = start + size;
            bands.push(NextGenBand::new(slice, start..end));
            start = end;
        }

        Self { bands }
    }

    fn compute(self) -> Vec<Pos2> {
        // use rayon to collect the outputs of each NextGenBand, which combined
        // results in the next generation (see merge step below)
        let alive_bands: Vec<Vec<Pos2>> = self
            .bands
            .into_par_iter()
            .map(|band| band.collect())
            .collect();

        // merge all NextGenBand outputs together
        // they may (probably) have duplicate values too
        let total_len: usize = alive_bands.iter().map(Vec::len).sum();
        let mut alive = Vec::with_capacity(total_len);
        for output in alive_bands {
            if output.is_empty() {
                continue;
            }
            let start = match alive.last() {
                Some(&last) => output.partition_point(|&pos| pos <= last),
                None => 0,
            };
            alive.extend_from_slice(&output[start..]);
        }

        alive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(x: i32, y: i32) -> Pos2 {
        Pos2 { x, y }
    }

    fn sorted(mut positions: Vec<Pos2>) -> Vec<Pos2> {
        positions.sort();
        positions.dedup();
        positions
    }

    #[test]
    fn block_is_stable() {
        let alive = sorted(vec![pos(1, 1), pos(2, 1), pos(1, 2), pos(2, 2)]);
        let mut game = GameOfLife::from_alive(alive.clone());

        game.next_generation();

        assert_eq!(game.take(), alive);
    }

    #[test]
    fn blinker_oscillates() {
        let start = sorted(vec![pos(1, 0), pos(1, 1), pos(1, 2)]);
        let mid = sorted(vec![pos(0, 1), pos(1, 1), pos(2, 1)]);
        let mut game = GameOfLife::from_alive(start.clone());

        game.next_generation();
        assert_eq!(game.alive, mid);

        game.next_generation();
        assert_eq!(game.alive, start);
    }

    #[test]
    fn parallel_matches_serial() {
        let alive = sorted(vec![
            pos(1, 0),
            pos(2, 1),
            pos(0, 2),
            pos(1, 2),
            pos(2, 2),
            pos(4, 4),
            pos(5, 4),
            pos(6, 4),
            pos(5, 5),
        ]);
        let mut serial = GameOfLife::from_alive(alive.clone());
        let mut parallel = GameOfLife::from_alive(alive);

        serial.next_generation();
        parallel.next_generation_parallel();

        assert_eq!(serial.take(), parallel.take());
    }

    #[test]
    fn window_filters_positions() {
        let alive = sorted(vec![pos(-1, 0), pos(0, 0), pos(1, 1), pos(2, 2)]);
        let game = GameOfLife::from_alive(alive);
        let window = game.window(pos(0, 0), pos(2, 2));

        let collected: Vec<Pos2> = window.iter().copied().collect();

        assert_eq!(collected, vec![pos(0, 0), pos(1, 1)]);
    }

    #[test]
    fn striped_handles_small_inputs() {
        let empty: Vec<Pos2> = Vec::new();
        let one = sorted(vec![pos(0, 0)]);
        let two = sorted(vec![pos(0, 0), pos(1, 0)]);

        let empty_next = StripedNextGen::with_bands(&empty, 8).compute();
        let one_next = StripedNextGen::with_bands(&one, 8).compute();
        let two_next = StripedNextGen::with_bands(&two, 8).compute();

        assert_eq!(empty_next, Vec::<Pos2>::new());
        assert_eq!(one_next, NextGen::new(&one).collect::<Vec<_>>());
        assert_eq!(two_next, NextGen::new(&two).collect::<Vec<_>>());
    }

    #[test]
    fn striped_band_boundary_matches_serial() {
        let alive = sorted(vec![
            pos(0, 0),
            pos(1, 0),
            pos(2, 0),
            pos(0, 1),
            pos(2, 1),
        ]);
        let serial = NextGen::new(&alive).collect::<Vec<_>>();
        let striped = StripedNextGen::with_bands(&alive, 2).compute();

        assert_eq!(serial, striped);
    }

    #[test]
    fn striped_randomized_matches_serial() {
        fn pseudo_positions(seed: u32, count: usize) -> Vec<Pos2> {
            let mut value = seed;
            let mut out = Vec::with_capacity(count);
            for _ in 0..count {
                value = value.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                let x = ((value >> 16) % 41) as i32 - 20;
                value = value.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                let y = ((value >> 16) % 41) as i32 - 20;
                out.push(Pos2 { x, y });
            }
            out
        }

        for seed in 1..=8 {
            let alive = sorted(pseudo_positions(seed, 64));
            let serial = NextGen::new(&alive).collect::<Vec<_>>();
            let striped = StripedNextGen::with_bands(&alive, 4).compute();

            assert_eq!(serial, striped, "seed {seed} failed");
        }
    }
}
