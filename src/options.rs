use std::time::Duration;

use cgolrs::Pos2;

pub struct Args {
    matches: getopts::Matches,
}

impl Args {
    fn new<T: AsRef<str>>(args: &[T]) -> Option<Self> {
        let mut opts = getopts::Options::new();
        opts.optflag("", "help", "print this help menu");
        opts.optflag("c", "console", "run in console mode");
        opts.optflag("t", "threads", "enables multi-threading");
        opts.optopt("o", "output", "output file", "FILE");
        opts.optopt("i", "input", "input file", "FILE");
        opts.optopt("w", "width", "set grid width", "WIDTH");
        opts.optopt("h", "height", "set grid height", "HEIGHT");
        opts.optopt("f", "fill", "set fill type", "TYPE");
        opts.optopt(
            "s",
            "sleep",
            "the amount of time to sleep between generations",
            "MILLIS",
        );
        opts.optopt("g", "gens", "max number of generations", "COUNT");
        opts.optopt("", "stats", "write stats csv to file", "FILE");

        let matches = opts.parse(args.iter().map(T::as_ref)).unwrap();
        if matches.opt_present("help") {
            println!("{}", opts.usage("usage: gol [options] [FILE]"));
            None
        } else {
            Some(Self { matches })
        }
    }
    pub fn from_env() -> Option<Self> {
        let env = std::env::args().collect::<Vec<_>>();
        Self::new(&env[1..])
    }

    fn width(&self) -> Option<i32> {
        self.matches.opt_get("width").unwrap()
    }
    fn height(&self) -> Option<i32> {
        self.matches.opt_get("height").unwrap()
    }

    pub fn console(&self) -> bool {
        self.matches.opt_present("console")
    }
    pub fn multithreading(&self) -> bool {
        self.matches.opt_present("threads")
    }

    pub fn generations(&self) -> usize {
        self.matches.opt_get("gens").unwrap().unwrap_or(usize::MAX) // kinda hacky way of saying "infinity"
    }
    pub fn sleep(&self) -> Option<Duration> {
        match self.matches.opt_get("sleep").unwrap() {
            Some(millis) => Some(Duration::from_millis(millis)),
            None if self.console() => Some(Duration::from_millis(100)),
            None => None,
        }
    }

    pub fn grid_size(&self) -> (i32, i32) {
        let default = if self.console() {
            let (cols, rows) = crossterm::terminal::size().unwrap();
            (cols as i32, rows as i32)
        } else {
            (500, 500)
        };

        (
            self.width().unwrap_or(default.0),
            self.height().unwrap_or(default.1),
        )
    }
    pub fn fill_mode(&self) -> FillMode {
        let mode_str = self.matches.opt_str("fill");
        FillMode::new(mode_str.as_deref().unwrap_or("random")).expect("valid fill mode string")
    }

    pub fn output_file(&self) -> Option<String> {
        self.matches.opt_str("output")
    }
    pub fn input_file(&self) -> Option<String> {
        self.matches.opt_str("input")
    }

    pub fn stats_file(&self) -> Option<String> {
        self.matches.opt_str("stats")
    }
}

pub enum FillMode {
    Random,
    Alternating,
    All,
    Empty,
}
impl FillMode {
    fn new<S: AsRef<str>>(s: S) -> Option<Self> {
        match s.as_ref() {
            "random" => Some(Self::Random),
            "alternating" => Some(Self::Alternating),
            "all" => Some(Self::All),
            "empty" => Some(Self::Empty),
            _ => None,
        }
    }

    fn reserve_size(&self, w: i32, h: i32) -> usize {
        let total = (w as usize) * (h as usize);
        match self {
            Self::Random => (total + 1) / 2,
            Self::Alternating => (total + 1) / 2,
            Self::All => total,
            Self::Empty => 0,
        }
    }
    fn fill_cell<R: rand::Rng>(&self, cell: Pos2, rng: &mut R) -> bool {
        match self {
            Self::Random => rng.random_bool(0.5),
            Self::Alternating => (cell.x + cell.y) % 2 == 0,
            Self::All => true,
            Self::Empty => false,
        }
    }
    pub fn create_alive(self, w: i32, h: i32) -> Vec<Pos2> {
        let mut alive = Vec::new();
        let reserve_size = self.reserve_size(w, h);
        if reserve_size == 0 {
            // reserve_size indicates this will produce no alive cells
            return alive;
        }

        let mut rng = rand::rng();
        alive.reserve(reserve_size);
        for y in 0..h {
            for x in 0..w {
                let cell = Pos2 { x, y };
                if self.fill_cell(cell, &mut rng) {
                    alive.push(cell);
                }
            }
        }
        alive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args_with_fill(fill: &str) -> Args {
        Args::new(&["--fill", fill]).expect("args with fill")
    }

    fn pos(x: i32, y: i32) -> Pos2 {
        Pos2 { x, y }
    }

    #[test]
    fn fill_mode_parses() {
        let args = args_with_fill("alternating");

        assert!(matches!(args.fill_mode(), FillMode::Alternating));
    }

    #[test]
    fn create_alive_all_fills_grid() {
        let alive = FillMode::All.create_alive(3, 2);

        let expected = vec![
            pos(0, 0),
            pos(1, 0),
            pos(2, 0),
            pos(0, 1),
            pos(1, 1),
            pos(2, 1),
        ];
        assert_eq!(alive, expected);
    }

    #[test]
    fn create_alive_empty_is_empty() {
        let alive = FillMode::Empty.create_alive(5, 4);

        assert!(alive.is_empty());
    }

    #[test]
    fn create_alive_alternating_uses_parity() {
        let alive = FillMode::Alternating.create_alive(3, 3);

        let expected = vec![
            pos(0, 0),
            pos(2, 0),
            pos(1, 1),
            pos(0, 2),
            pos(2, 2),
        ];
        assert_eq!(alive, expected);
    }

    #[test]
    fn create_alive_random_is_within_bounds() {
        let w = 4;
        let h = 3;
        let alive = FillMode::Random.create_alive(w, h);

        assert!(alive.iter().all(|p| p.x >= 0 && p.y >= 0 && p.x < w && p.y < h));
    }
}
