use std::{
    io,
    thread,
    time::Instant,
};

mod console;
mod enc;
mod engine;
mod options;
mod pos;

use pos::Pos2;

fn args_to_alive(args: &options::Args) -> Vec<Pos2> {
    if let Some(file_name) = args.input_file() {
        let decoder = enc::RunLengthEncoded::default();
        let encoded_str = std::fs::read_to_string(file_name).unwrap();
        return enc::PositionEncoder::decode(decoder, &encoded_str);
    }

    // setup the alive cells based on args
    let (grid_w, grid_h) = args.grid_size();
    let mut alive = Vec::new();
    for y in 0..grid_h {
        for x in 0..grid_w {
            if args.fill_is_alive(x, y) {
                alive.push(Pos2 {
                    x: x as i32,
                    y: y as i32,
                });
            }
        }
    }
    alive
}

fn main() -> io::Result<()> {
    let Some(args) = options::Args::from_env() else {
        panic!("invalid arguments");
    };

    let alive = args_to_alive(&args);
    println!("alive: {}", alive.len());

    // setup the engine and reporting metrics
    let mut game = engine::GameOfLife::from_alive(alive);
    let sleep = args.sleep();
    let mut total_gens = 0;
    let mut report_gens = 0;
    let mut last_report = Instant::now();

    let mut console = if args.console() {
        Some(console::ConsoleRender::new()?)
    } else {
        None
    };
    'generations: for _ in 0..args.generations() {
        // render the console if in console mode
        if let Some(ref mut console) = console {
            while let Some(cmd) = console.poll_events()? {
                match cmd {
                    console::ConsoleCommand::Exit => break 'generations,
                    _ => {},
                }
            }
            console.render(&game)?;
        }

        // report metrics every 500ms or always if in console mode
        if last_report.elapsed().as_millis() >= 500 || console.is_some() {
            total_gens += report_gens;

            // calculate gens/sec and reset reporting numbers
            let gens_per_sec = report_gens as f64 / last_report.elapsed().as_secs_f64();
            report_gens = 0;
            last_report = Instant::now();

            let report = format!(
                "{:.02}gen/s gens:{}, alive:{}",
                gens_per_sec,
                total_gens,
                game.alive_count()
            );
            if let Some(ref mut console) = console {
                console.set_report(report);
            } else {
                println!("{}", report);
            }
        }

        // compute the next generation
        game.next_generation();
        report_gens += 1;
        if let Some(time) = sleep {
            thread::sleep(time);
        }
    }
    std::mem::drop(console);

    if let Some(file_name) = args.output_file() {
        let encoder = enc::RunLengthEncoded::default().set_name("cgol_sim generated pattern");
        let encoded_game = enc::PositionEncoder::encode(encoder, &game.take());
        std::fs::write(file_name, encoded_game).expect("write encoded game to file");
    }

    Ok(())
}
