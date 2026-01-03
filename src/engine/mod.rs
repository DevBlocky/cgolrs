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

        // use rayon to collect the outputs of each NextGenBand, which combined
        // results in the next generation (see merge step below)
        let band_outputs: Vec<Vec<Pos2>> =
            NextGenBand::create_bands(&self.alive, rayon::current_num_threads())
                .into_par_iter()
                .map(|band_iter| band_iter.collect())
                .collect();

        // merge all NextGenBand outputs together
        // they may (probably) have duplicate values too
        let total_len: usize = band_outputs.iter().map(Vec::len).sum();
        let mut next = Vec::with_capacity(total_len);
        for output in band_outputs {
            if output.is_empty() {
                continue;
            }
            let start = match next.last() {
                Some(&last) => output.partition_point(|&pos| pos <= last),
                None => 0,
            };
            next.extend_from_slice(&output[start..]);
        }

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
    fn create_bands(slice: &'a [Pos2], n: usize) -> Vec<NextGenBand<'a>> {
        if slice.is_empty() || n == 0 {
            return Vec::new();
        }

        let n = n.min(slice.len());
        let base = slice.len() / n;
        let remainder = slice.len() % n;
        let mut bands = Vec::with_capacity(n);
        let mut start = 0;
        for i in 0..n {
            let size = base + usize::from(i < remainder);
            let end = start + size;
            bands.push(Self::new(slice, start..end));
            start = end;
        }
        bands
    }

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
