use super::GameOfLife;
use crate::Pos2;

pub struct GameEngineWindow<'a> {
    tl: Pos2,
    br: Pos2,
    engine: &'a GameOfLife,
}
impl<'a> GameEngineWindow<'a> {
    pub fn new(engine: &'a GameOfLife, top_left: Pos2, bottom_right: Pos2) -> Self {
        Self {
            tl: top_left,
            br: bottom_right,
            engine,
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Pos2> {
        let rx = self.tl.x..self.br.x;
        let ry = self.tl.y..self.br.y;
        self.engine
            .alive
            .iter()
            .filter(move |pos| rx.contains(&pos.x) && ry.contains(&pos.y))
    }
}

impl<'a> std::fmt::Display for GameEngineWindow<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut last = self.tl - Pos2 { x: 1, y: 0 };
        for alive in self.iter() {
            // determine the number of lines to print
            let lines = alive.y - last.y;
            // determine the number of padding spaces to print
            let padding = match lines {
                0 => alive.x - last.x - 1,
                _ => alive.x - self.tl.x,
            };
            write!(
                f,
                "{0:\n<1$}{0: <2$}█",
                "", lines as usize, padding as usize
            )?;
            last = *alive;
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
    fn iter_respects_bounds() {
        let alive = sorted(vec![pos(0, 0), pos(1, 1), pos(2, 2)]);
        let game = GameOfLife::from_alive(alive);
        let window = GameEngineWindow::new(&game, pos(0, 0), pos(2, 2));

        let collected: Vec<Pos2> = window.iter().copied().collect();

        assert_eq!(collected, vec![pos(0, 0), pos(1, 1)]);
    }
}
