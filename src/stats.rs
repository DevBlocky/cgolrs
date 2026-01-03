use std::time::Instant;

pub trait Recorder {
    type Str: AsRef<str>;

    fn record(&mut self, alive: usize);

    fn has_report(&self, interactive: bool) -> bool;
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

    fn has_report(&self, interactive: bool) -> bool {
        interactive || self.last_report.elapsed().as_millis() >= 500
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
    gens: usize,
    data: Vec<(u128, usize)>,
    last: Instant,
}
impl CsvRecord {
    pub fn new(alive: usize) -> Self {
        Self {
            inner: SimpleRecord::new(alive),
            gens: 0,
            data: Vec::new(),
            last: Instant::now(),
        }
    }
}
impl Recorder for CsvRecord {
    type Str = String; // doesnt matter in this case

    fn record(&mut self, alive: usize) {
        let delta = self.last.elapsed().as_micros();
        self.last = Instant::now();
        self.gens += 1;

        self.data.push((delta, alive));
        self.inner.record(alive);
    }

    // never has a console report
    fn has_report(&self, interactive: bool) -> bool {
        self.inner.has_report(interactive)
    }
    fn report(&mut self) -> Self::Str {
        self.inner.report()
    }
}
impl Drop for CsvRecord {
    fn drop(&mut self) {
        use std::{fs, io::{self, Write}};

        let file = fs::File::create("perf.csv").expect("create perf.csv");
        let mut file = io::BufWriter::new(file);

        file.write_all(b"gen,delta_t,alive\n").unwrap();
        for (i, (delta, alive)) in self.data.iter().enumerate() {
            let line = format!("{},{},{}\n", i, delta, alive);
            file.write_all(line.as_bytes()).unwrap();
        }
        file.flush().unwrap();
    }
}
