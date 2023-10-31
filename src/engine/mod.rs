mod index;
mod scan;
mod window;

use self::scan::MultiRowPosCursor;
pub use self::window::GameEngineWindow;
use crate::Pos2;

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
        let next = NextGeneration::new(&self.alive).collect::<Vec<_>>();
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

struct NextGeneration<'a> {
    cursor: MultiRowPosCursor<'a>,
}
impl<'a> NextGeneration<'a> {
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
impl Iterator for NextGeneration<'_> {
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
