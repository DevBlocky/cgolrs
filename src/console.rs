use crate::{engine::GameOfLife, Pos2};
use crossterm::{
    cursor,
    event::{self, KeyCode, KeyEvent, KeyModifiers},
    queue, terminal,
    execute,
};
use std::io;

pub enum ConsoleCommand {
    Exit,
    Handled,
}

pub struct ConsoleRender {
    tl: Pos2,
    report: String,
}
impl ConsoleRender {
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), cursor::Hide)?;
        Ok(Self {
            tl: Pos2::default(),
            report: String::new(),
        })
    }

    pub fn render(&self, game: &GameOfLife) -> io::Result<()> {
        let (cols, rows) = terminal::size()?;
        let br = self.tl
            + Pos2 {
                x: cols as i32,
                y: rows as i32,
            };
        let mut stdout = io::stdout();
        queue!(stdout, terminal::Clear(terminal::ClearType::All))?;
        for cell in game.window(self.tl, br).iter() {
            let cell = *cell - self.tl;
            queue!(stdout, cursor::MoveTo(cell.x as u16, cell.y as u16))?;
            io::Write::write_all(&mut stdout, b"\xE2\x96\x88")?;
        }

        // write footer
        queue!(stdout, cursor::MoveTo(0, rows))?;
        io::Write::write_all(&mut stdout, self.report.as_bytes())?;

        io::Write::flush(&mut stdout)
    }

    pub fn poll_events(&mut self) -> io::Result<Option<ConsoleCommand>> {
        // make sure event is preset for us to take
        if !event::poll(std::time::Duration::from_secs(0))? {
            return Ok(None);
        }

        let mut outp = Ok(Some(ConsoleCommand::Handled));
        match event::read()? {
            // CTRL+C
            event::Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => {
                outp = Ok(Some(ConsoleCommand::Exit));
            }
            // arrows to move grid
            event::Event::Key(
                ev @ KeyEvent {
                    code: KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right,
                    ..
                },
            ) => match ev.code {
                KeyCode::Up => self.tl.y -= 1,
                KeyCode::Down => self.tl.y += 1,
                KeyCode::Left => self.tl.x -= 1,
                KeyCode::Right => self.tl.x += 1,
                _ => {}
            },
            _ => {}
        }
        outp
    }

    pub fn set_report(&mut self, report: String) {
        self.report = report;
    }
}
impl Drop for ConsoleRender {
    fn drop(&mut self) {
        // if we can enable it, we should be able to disable it
        terminal::disable_raw_mode().expect("disable raw mode");
        execute!(io::stdout(), cursor::Show).expect("enable cursor");
    }
}
