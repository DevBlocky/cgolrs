use std::{io, thread};

mod console;
mod options;
mod stats;

use crate::stats::Recorder;
use cgolrs::{Pos2, enc, engine};

fn args_to_alive(args: &options::Args) -> Vec<Pos2> {
    if let Some(file_name) = args.input_file() {
        let codec = enc::RunLengthEncoded::default();
        let encoded_str = std::fs::read_to_string(file_name).unwrap();
        return enc::Codec::decode(codec, &encoded_str);
    }

    // setup the alive cells based on args
    let (grid_w, grid_h) = args.grid_size();
    let fill = args.fill_mode();
    fill.create_alive(grid_w, grid_h)
}

fn main() -> io::Result<()> {
    let Some(args) = options::Args::from_env() else {
        panic!("invalid arguments");
    };

    let alive = args_to_alive(&args);
    println!("alive: {}", alive.len());

    // setup the engine and reporting metrics
    let mut game = engine::GameOfLife::from_alive(alive);
    let mut console = args
        .console()
        .then(|| console::ConsoleRender::new())
        .transpose()?;
    let mut stats = stats::SwitchRecorder::new(game.alive_count(), args.stats_file().is_some());
    let sleep = args.sleep();
    let parallel = args.multithreading();

    // main loop
    'generations: for _ in 0..args.generations() {
        // render the console if in console mode
        if let Some(ref mut console) = console {
            while let Some(cmd) = console.poll_events()? {
                match cmd {
                    console::ConsoleCommand::Exit => break 'generations,
                    _ => {}
                }
            }
            console.render(&game)?;
        }

        // report metrics every 500ms or always if in console mode
        if console.is_some() || stats.has_report() {
            let report = stats.report();
            if let Some(ref mut console) = console {
                console.set_report(report);
            } else {
                println!("{}", report);
            }
        }

        // compute the next generation
        if parallel {
            game.next_generation_parallel();
        } else {
            game.next_generation();
        }
        stats.record(game.alive_count());
        if let Some(time) = sleep {
            thread::sleep(time);
        }
    }
    std::mem::drop(console);

    // save output file
    if let Some(file_name) = args.output_file() {
        let encoder = enc::RunLengthEncoded::default().set_name("cgol_sim generated pattern");
        let encoded_game = enc::Codec::encode(encoder, &game.take());
        std::fs::write(file_name, encoded_game).expect("write encoded game to file");
    }
    // save stats file
    if let Some(stats_file) = args.stats_file() {
        stats.save(stats_file).expect("write stats csv to file");
    }

    Ok(())
}
