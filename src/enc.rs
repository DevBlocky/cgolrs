use crate::pos::Pos2;

pub trait PositionEncoder {
    fn encode(self, positions: &[Pos2]) -> String;
    fn decode(self, value: &str) -> Vec<Pos2>;
}

struct RunEncoder {
    sequence: String,
    line_len: usize,
    max_line_len: usize,
}
impl RunEncoder {
    fn new(max_line_len: usize) -> Self {
        Self {
            sequence: String::new(),
            line_len: 0,
            max_line_len,
        }
    }

    fn push_run(&mut self, run: i32, c: char) {
        let append = match run {
            0 => String::new(),
            1 => c.to_string(),
            n => format!("{}{}", n, c),
        };
        if self.line_len + append.len() > self.max_line_len {
            self.sequence.push('\n');
            self.line_len = 0;
        }
        self.line_len += append.len();
        self.sequence.push_str(&append);
    }

    pub fn end(mut self) -> String {
        self.sequence.push('!');
        self.sequence
    }
}

pub struct RunLengthEncoded {
    name: Option<String>,
    header: bool,
}
impl RunLengthEncoded {
    pub fn set_name<T: AsRef<str>>(mut self, name: T) -> Self {
        self.name = Some(name.as_ref().to_owned());
        self
    }

    fn encode_header(&self) -> String {
        let mut header = String::new();
        if !self.header {
            return header;
        }
        if let Some(name) = &self.name {
            header.push_str(&format!("#N {}\n", name));
        }
        header.push_str("x = 0, y = 0, rule = 23/3");
        header
    }
    fn encode_cells(&self, alive_cells: &[Pos2]) -> String {
        // top-left
        let tl = Pos2 {
            x: alive_cells.iter().map(|p| p.x).min().unwrap_or_default(),
            // because the cells are sorted, the first cells will always have the lowest y-value
            y: alive_cells.first().map(|p| p.y).unwrap_or_default(),
        };

        let mut last = tl - Pos2 { x: 1, y: 0 };
        let mut alive_run = 0;
        let mut seq = RunEncoder::new(70);
        for pos in alive_cells {
            // if we're one ahead of the last, then only increment the run
            if last.y == pos.y && (last.x + 1) == pos.x {
                alive_run += 1;
                last = *pos;
                continue;
            }

            let lines_run = pos.y - last.y;
            let dead_run = match lines_run {
                0 => pos.x - last.x - 1,
                _ => pos.x - tl.x,
            };
            // NOTE: order matters!
            seq.push_run(alive_run, 'o');
            seq.push_run(lines_run, '$');
            seq.push_run(dead_run, 'b');

            alive_run = 1;
            last = *pos;
        }

        seq.push_run(alive_run, 'o');
        seq.end()
    }
}
impl Default for RunLengthEncoded {
    fn default() -> Self {
        Self {
            name: None,
            header: true,
        }
    }
}

impl PositionEncoder for RunLengthEncoded {
    fn encode(self, cells: &[Pos2]) -> String {
        format!("{}\n{}\n", self.encode_header(), self.encode_cells(cells))
    }

    fn decode(self, value: &str) -> Vec<Pos2> {
        let re = regex::Regex::new(r"(\d*)([bo$!])").unwrap();

        let mut alive = Vec::new();
        let mut cursor = Pos2 { x: 0, y: 0 };
        'lines_loop: for mut line in value.split("\n") {
            if let Some(i) = line.find('#') {
                line = &line[..i];
            }

            for (_, [run_str, state]) in re.captures_iter(line).map(|x| x.extract()) {
                let run = run_str.parse::<i32>().unwrap_or(1);
                match state {
                    "!" => break 'lines_loop,
                    "o" => {
                        for _ in 0..run {
                            alive.push(cursor);
                            cursor.x += 1;
                        }
                    }
                    "b" => cursor.x += run,
                    "$" => {
                        cursor.x = 0;
                        cursor.y += run;
                    }
                    _ => unreachable!(),
                }
            }
        }

        alive
    }
}
