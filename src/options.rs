use std::time::Duration;

pub struct Args {
    matches: getopts::Matches,
}

impl Args {
    fn new<T: AsRef<str>>(args: &[T]) -> Option<Self> {
        let mut opts = getopts::Options::new();
        opts.optflag("", "help", "print this help menu");
        opts.optflag("c", "console", "run in console mode");
        opts.optopt("o", "output", "output file", "FILE");
        opts.optopt("i", "input", "input file", "FILE");
        opts.optopt("w", "width", "set grid width", "WIDTH");
        opts.optopt("h", "height", "set grid height", "HEIGHT");
        opts.optopt("f", "fill", "set fill type", "TYPE");
        opts.optopt("t", "threads", "number of worker threads", "COUNT");
        opts.optopt(
            "s",
            "sleep",
            "the amount of time to sleep between generations",
            "MILLIS",
        );
        opts.optopt("g", "gens", "max number of generations", "COUNT");

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
    fn fill(&self) -> Option<String> {
        self.matches.opt_str("fill")
    }

    pub fn console(&self) -> bool {
        self.matches.opt_present("console")
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
    pub fn threads(&self) -> usize {
        let auto_threads = std::thread::available_parallelism()
            .map(|value| value.get())
            .unwrap_or(1);
        match self.matches.opt_get("threads").unwrap() {
            Some(0) => auto_threads,
            Some(value) => value,
            None => 1,
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
    pub fn fill_is_alive(&self, x: i32, y: i32) -> bool {
        use rand::prelude::*;

        let f = self.fill();
        match f.as_deref().unwrap_or("random") {
            "random" => rand::rng().random_bool(0.5),
            "alternating" => (x + y) % 2 == 0,
            "all" => true,
            "empty" => false,
            _ => panic!("invalid fill type"),
        }
    }

    pub fn output_file(&self) -> Option<String> {
        self.matches.opt_str("output")
    }
    pub fn input_file(&self) -> Option<String> {
        self.matches.opt_str("input")
    }
}
