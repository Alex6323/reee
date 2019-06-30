//! Game-of-life reference implementation
#![allow(dead_code)]

mod common;

use common::gol::*;

use std::time::Instant;

use crossterm::{
    cursor,
    style,
    terminal,
    ClearType,
    Color,
};
use rand::Rng;
use structopt::StructOpt;

const UPDATE_INTERVAL: u64 = 50;
const WIDTH: usize = 80;
const HEIGHT: usize = 50;

#[derive(StructOpt)]
struct Args {
    #[structopt(subcommand)]
    universe: Subcommand,
}

#[derive(StructOpt)]
enum Subcommand {
    #[structopt(name = "random", about = "Random universe")]
    Random,

    #[structopt(name = "carlos", about = "Carlos universe :)")]
    Carlos,

    #[structopt(name = "glider_gun", about = "Glider gun")]
    GliderGun,
}
fn main() {
    let args = Args::from_args();
    let universe = match args.universe {
        Subcommand::Random => random(),
        Subcommand::Carlos => carlos(),
        Subcommand::GliderGun => glider_gun(),
    };
    run_gol(universe);
}

fn run_gol(mut u: Universe) {
    terminal().clear(ClearType::All).unwrap();

    let cursor = cursor();

    print_info("Running Game-Of-Life reference implementation (after 3s)...\n");

    cursor.save_position().unwrap();

    for i in 0.. {
        println!(
            "width={} height={} {}{}ms",
            u.width, u.height, "\u{2764}", UPDATE_INTERVAL
        );
        print!("{}", u);

        if i == 0 {
            std::thread::sleep(std::time::Duration::from_millis(3000));
        } else {
            std::thread::sleep(std::time::Duration::from_millis(UPDATE_INTERVAL));
        }

        let start = Instant::now();
        u.next_gen();
        let stop = start.elapsed();
        println!(
            "#{} {}{:02}s:{:09}ns",
            i,
            "\u{23f1}",
            stop.as_secs(),
            stop.subsec_nanos()
        );
        cursor.reset_position().unwrap();
    }
}

pub fn print_info(info: &str) {
    println!("{}", style(info.to_string()).with(Color::Yellow));
}

fn carlos() -> Universe {
    use common::luts::CARLOS;

    let carlos = CARLOS
        .chars()
        .filter_map(|c| match c.to_string().parse::<u8>() {
            Ok(s) => Some(s == 1),
            Err(_) => None,
        })
        .collect::<Vec<_>>();

    let mut u = Universe::new(148, 70);

    for (i, is_alive) in carlos.into_iter().enumerate() {
        let (x, y) = u.get_position(i);
        if is_alive {
            u.set_alive(x, y);
        }
    }

    u
}

fn random() -> Universe {
    let mut rng = rand::thread_rng();

    let mut u = Universe::new(WIDTH, HEIGHT);

    for i in 0..WIDTH * HEIGHT {
        let alive = rng.gen::<bool>();
        let (x, y) = u.get_position(i);
        if alive {
            u.set_alive(x, y)
        }
    }

    u
}

fn spinner() -> Universe {
    let mut gol = Universe::new(WIDTH, HEIGHT);

    gol.set_alive(2, 1);
    gol.set_alive(2, 2);
    gol.set_alive(2, 3);

    gol
}

fn glider_gun() -> Universe {
    let mut gol = Universe::new(WIDTH, HEIGHT);

    gol.set_alive(1, 5);
    gol.set_alive(2, 5);
    gol.set_alive(1, 6);
    gol.set_alive(2, 6);
    gol.set_alive(11, 5);
    gol.set_alive(11, 6);
    gol.set_alive(11, 7);
    gol.set_alive(12, 8);
    gol.set_alive(13, 9);
    gol.set_alive(14, 9);
    gol.set_alive(12, 4);
    gol.set_alive(13, 3);
    gol.set_alive(14, 3);
    gol.set_alive(15, 6);
    gol.set_alive(16, 4);
    gol.set_alive(16, 8);
    gol.set_alive(17, 7);
    gol.set_alive(17, 6);
    gol.set_alive(17, 5);
    gol.set_alive(18, 6);
    gol.set_alive(21, 3);
    gol.set_alive(21, 4);
    gol.set_alive(21, 5);
    gol.set_alive(22, 3);
    gol.set_alive(22, 4);
    gol.set_alive(22, 5);
    gol.set_alive(23, 2);
    gol.set_alive(23, 6);
    gol.set_alive(25, 1);
    gol.set_alive(25, 2);
    gol.set_alive(25, 6);
    gol.set_alive(25, 7);
    gol.set_alive(35, 3);
    gol.set_alive(35, 4);
    gol.set_alive(36, 3);
    gol.set_alive(36, 4);

    gol
}
