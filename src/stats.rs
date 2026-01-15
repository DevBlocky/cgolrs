use std::time::Instant;

pub trait Recorder {
    type Str: AsRef<str>;

    fn record(&mut self, alive: usize);

    fn has_report(&self) -> bool;
    fn report(&mut self) -> Self::Str;
}

pub struct SimpleRecord {
    gens: usize,
    alive: usize,
    gens_in_report: usize,
    last_report: Instant,
}
impl SimpleRecord {
    pub fn new(alive: usize) -> Self {
        Self {
            gens: 0,
            alive,
            gens_in_report: 0,
            last_report: Instant::now(),
        }
    }
}
impl Recorder for SimpleRecord {
    type Str = String;

    fn record(&mut self, alive: usize) {
        self.gens += 1;
        self.gens_in_report += 1;
        self.alive = alive;
    }

    fn has_report(&self) -> bool {
        self.last_report.elapsed().as_millis() >= 500
    }
    fn report(&mut self) -> Self::Str {
        let gens_per_sec = self.gens_in_report as f64 / self.last_report.elapsed().as_secs_f64();
        // reset stats for next report
        self.last_report = Instant::now();
        self.gens_in_report = 0;

        format!(
            "{:.02}gen/s gens:{}, alive:{}",
            gens_per_sec, self.gens, self.alive
        )
    }
}

pub struct CsvRecord {
    inner: SimpleRecord,
    data: Vec<(u128, usize)>,
    last: Instant,
}
impl CsvRecord {
    pub fn new(alive: usize) -> Self {
        Self {
            inner: SimpleRecord::new(alive),
            data: Vec::new(),
            last: Instant::now(),
        }
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        use std::{
            fs,
            io::{self, Write},
        };

        let file = fs::File::create(path)?;
        let mut file = io::BufWriter::new(file);

        file.write_all(b"gen,delta_t,alive\n")?;
        for (i, (delta, alive)) in self.data.iter().enumerate() {
            let line = format!("{},{},{}\n", i, delta, alive);
            file.write_all(line.as_bytes())?;
        }
        file.flush()
    }
}
impl Recorder for CsvRecord {
    type Str = <SimpleRecord as Recorder>::Str;

    fn record(&mut self, alive: usize) {
        let delta = self.last.elapsed().as_micros();
        self.last = Instant::now();

        self.data.push((delta, alive));
        self.inner.record(alive);
    }

    // never has a console report
    fn has_report(&self) -> bool {
        self.inner.has_report()
    }
    fn report(&mut self) -> Self::Str {
        self.inner.report()
    }
}

pub enum SwitchRecorder {
    Csv(CsvRecord),
    Simple(SimpleRecord),
}
impl SwitchRecorder {
    pub fn new(alive: usize, csv: bool) -> Self {
        if csv {
            Self::Csv(CsvRecord::new(alive))
        } else {
            Self::Simple(SimpleRecord::new(alive))
        }
    }
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        match self {
            Self::Csv(r) => r.save(path),
            _ => panic!("cannot save statistics if not CsvRecord type"),
        }
    }
}
impl Recorder for SwitchRecorder {
    type Str = String;

    fn record(&mut self, alive: usize) {
        match self {
            Self::Csv(r) => r.record(alive),
            Self::Simple(r) => r.record(alive),
        }
    }
    fn has_report(&self) -> bool {
        match self {
            Self::Csv(r) => r.has_report(),
            Self::Simple(r) => r.has_report(),
        }
    }
    fn report(&mut self) -> Self::Str {
        match self {
            Self::Csv(r) => r.report(),
            Self::Simple(r) => r.report(),
        }
    }
}
