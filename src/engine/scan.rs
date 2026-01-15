use crate::pos::Pos2;

/// A cursor over a slice of ordered (not strictly sequential) [`Pos2`]s
///
/// [`PosCursor`] acts as a sort of grid iterator, where each position in the
/// grid is either present (in the slice) or absent (not in the slice).
/// It works by horizontally scanning a row's x-values, and keeping a buffer
/// of whether the last 8 are present or absent.
///
/// [`PosCursor::seek`] is required to be called to jump to another row in
/// the meta-grid, or else it will continue to scan the same row
///
/// [`PosCursor::seek`]: #method.seek
#[derive(Debug)]
struct PosCursor<'a> {
    slice: &'a [Pos2],
    next_idx: usize,
    cursor: Pos2,

    buffer: u8,
}

impl<'a> PosCursor<'a> {
    /// Creates a new [`PosCursor`] over the given slice, starting at the given position
    fn new(slice: &'a [Pos2], cursor: Pos2) -> Self {
        let mut value = Self {
            slice,
            // these fields will be overwritten by `seek`
            next_idx: 0,
            cursor: Pos2::default(),
            buffer: 0,
        };
        value.seek(cursor);
        value
    }

    /// Increments this cursor to the next x-value in the meta-grid
    ///
    /// ## Returns
    /// The bit buffer for whether the last 8 [`Pos2::x`] values we're present
    /// in the slice
    ///
    /// See [`PosCursor::bit_buffer`] for more information
    fn next(&mut self) -> u8 {
        self.buffer <<= 1;
        self.cursor.x += 1;

        if self.slice.get(self.next_idx) == Some(&self.cursor) {
            self.next_idx += 1;
            self.buffer |= 1;
        }
        self.buffer
    }

    /// Seeks to a specific position in the meta-grid
    ///
    /// ## Returns
    /// The bit buffer for whether the last 8 [`Pos2::x`] values we're present
    /// in the slice
    ///
    /// See [`PosCursor::bit_buffer`] for more information
    fn seek(&mut self, cursor: Pos2) -> u8 {
        self.next_idx = match self.slice.get(self.next_idx) {
            // keep the same idx if the cursor is in-between the previous and next positions
            Some(&next) if self.cursor <= cursor && next > cursor => self.next_idx,
            // if the cursor is the next position, only increment the idx
            Some(&next) if next == cursor => self.next_idx + 1,
            // we have no clue what the next idx could be, so just binary search tha jawn
            _ => match self.slice.binary_search(&cursor) {
                Ok(i) => i + 1, // +1 because we want the _next_ index from the cursor
                Err(i) => i,
            },
        };
        self.cursor = cursor;

        self.reset_buffer();
        self.buffer
    }
    fn reset_buffer(&mut self) {
        self.buffer = 0;

        // use the slice to figure out the states of the 8 bits
        for i in 0..self.next_idx {
            let idx = self.next_idx - i - 1;
            let pos = self.slice[idx];
            let offset = self.cursor.x - pos.x;

            // offset >= 8 is out of the scope of an 8-bit buffer
            if pos.y != self.cursor.y || offset >= 8 {
                break;
            }
            debug_assert!(offset >= 0, "negative offset (out of bounds)");
            self.buffer |= 1 << offset;
        }
    }

    /// The state of the last 8 positions as bits in a [`u8`]
    ///
    /// Each bit represents whether the position is present in the slice.
    /// The state can be determined by a little bit math:
    /// ```rust,ignore
    /// let buffer = cursor.bit_buffer();
    /// let state1 = buffer & (1 << 0) != 0; // this is the state at the cursor
    /// let state2 = buffer & (1 << 1) != 0; // this is the state right behind the cursor
    /// let state3 = buffer & (1 << 2) != 0; // two behind the cursor
    /// // ...etc
    /// ```
    #[inline]
    fn bit_buffer(&self) -> u8 {
        self.buffer
    }

    /// The position for the next present [`Pos2`] in the slice
    #[inline]
    fn next_present(&self) -> Option<Pos2> {
        self.slice.get(self.next_idx).copied()
    }
    /// The current position of the cursor
    #[inline]
    fn cursor(&self) -> Pos2 {
        self.cursor
    }
}

pub struct MultiRowPosCursor<'a> {
    cursors: Vec<PosCursor<'a>>,
    buffers: Vec<u8>,
}
impl<'a> MultiRowPosCursor<'a> {
    /// Generates an iterator of y-offsets for the cursors
    ///
    /// Since each [`PosCursor`] is a cursor over a single line, this iterator will
    /// generate the y-offsets required for each cursor to be on a separate line
    ///
    /// The last (or bottom-most) offset will have an offset of 0, and all above
    /// will be negative y-values
    fn offset_iter(n: usize) -> impl Iterator<Item = Pos2> {
        (0..n).rev().map(|y_offset| Pos2 {
            x: 0,
            y: -(y_offset as i32),
        })
    }

    pub fn new(slice: &'a [Pos2], n_cursors: usize) -> Self {
        let start = slice.get(0).copied().unwrap_or_default();

        // create the cursors, with the bottom-most cursor being last but having 0 y-offset from the start
        let cursors: Vec<PosCursor<'_>> = Self::offset_iter(n_cursors)
            .map(|offset| PosCursor::new(slice, start + offset))
            .collect();
        let buffers = cursors.iter().map(PosCursor::bit_buffer).collect();

        Self { cursors, buffers }
    }

    #[inline]
    pub fn buffers(&self) -> &[u8] {
        &self.buffers
    }

    pub fn next(&mut self) -> &[u8] {
        for (i, cursor) in self.cursors.iter_mut().enumerate() {
            self.buffers[i] = cursor.next();
        }
        self.buffers()
    }

    pub fn seek(&mut self, pos: Pos2) -> &[u8] {
        // seek to the closest next position for every cursor and store the bit buffer
        for (i, (offset, cursor)) in Self::offset_iter(self.cursors.len())
            .zip(self.cursors.iter_mut())
            .enumerate()
        {
            self.buffers[i] = cursor.seek(pos + offset);
        }
        self.buffers()
    }
    pub fn seek_closest(&mut self) -> Option<&[u8]> {
        // find the next closest present position over all cursors
        let closest_next = Self::offset_iter(self.cursors.len())
            .zip(self.cursors.iter())
            .map(|(offset, cursor)| cursor.next_present().map(|present| present - offset))
            .flatten()
            .min()?;
        Some(self.seek(closest_next))
    }

    /// Returns the cursor position of the bottom most cursor
    #[inline]
    pub fn cursor(&self) -> Pos2 {
        self.cursors
            .last()
            .map(PosCursor::cursor)
            .unwrap_or_default()
    }
}
impl<'a> std::fmt::Display for MultiRowPosCursor<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for pc in &self.cursors {
            write!(f, "{:08b} ", pc.bit_buffer())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos::Pos2;

    fn pos(x: i32, y: i32) -> Pos2 {
        Pos2 { x, y }
    }

    fn sorted(mut positions: Vec<Pos2>) -> Vec<Pos2> {
        positions.sort();
        positions.dedup();
        positions
    }

    #[test]
    fn pos_cursor_tracks_bit_buffer() {
        let slice = sorted(vec![pos(0, 0), pos(2, 0), pos(1, 1)]);
        let mut cursor = PosCursor::new(&slice, pos(0, 0));

        assert_eq!(cursor.bit_buffer(), 0b1);
        assert_eq!(cursor.next_present(), Some(pos(2, 0)));

        let buffer = cursor.next();
        assert_eq!(buffer, 0b10);
        assert_eq!(cursor.cursor(), pos(1, 0));
        assert_eq!(cursor.next_present(), Some(pos(2, 0)));

        let buffer = cursor.next();
        assert_eq!(buffer, 0b101);
        assert_eq!(cursor.cursor(), pos(2, 0));
        assert_eq!(cursor.next_present(), Some(pos(1, 1)));

        let buffer = cursor.seek(pos(0, 1));
        assert_eq!(buffer, 0);
        assert_eq!(cursor.cursor(), pos(0, 1));
    }

    #[test]
    fn multi_row_seek_closest_advances() {
        let slice = sorted(vec![pos(0, 0), pos(2, 0), pos(1, 1)]);
        let mut cursor = MultiRowPosCursor::new(&slice, 1);

        assert_eq!(cursor.cursor(), pos(0, 0));

        cursor.seek_closest().expect("closest seek");
        assert_eq!(cursor.cursor(), pos(2, 0));

        cursor.seek_closest().expect("closest seek");
        assert_eq!(cursor.cursor(), pos(1, 1));
    }

    #[test]
    fn multi_row_buffers_update_on_next() {
        let slice = sorted(vec![pos(0, 0), pos(1, 0), pos(0, 1)]);
        let mut cursor = MultiRowPosCursor::new(&slice, 2);

        cursor.seek(pos(0, 1));
        assert_eq!(cursor.buffers(), &[0b1, 0b1]);

        cursor.next();
        assert_eq!(cursor.buffers(), &[0b11, 0b10]);
    }

    #[test]
    fn multi_row_seek_closest_empty_slice_is_none() {
        let slice: Vec<Pos2> = Vec::new();
        let mut cursor = MultiRowPosCursor::new(&slice, 3);

        assert_eq!(cursor.buffers(), &[0, 0, 0]);
        assert!(cursor.seek_closest().is_none());
    }

    #[test]
    fn pos_cursor_seek_between_positions() {
        let slice = sorted(vec![pos(0, 0), pos(2, 0), pos(1, 1)]);
        let mut cursor = PosCursor::new(&slice, pos(0, 0));

        let buffer = cursor.seek(pos(1, 0));
        assert_eq!(buffer, 0b10);
        assert_eq!(cursor.next_present(), Some(pos(2, 0)));
        assert_eq!(cursor.cursor(), pos(1, 0));
    }
}
