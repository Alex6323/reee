//! Game-of-life reference implementation

use rand::Rng;
use std::time::Instant;

use crossterm::{
    cursor,
    style,
    terminal,
    ClearType,
    Color,
};

const UPDATE_INTERVAL: u64 = 50;
const WIDTH: usize = 50;
const HEIGHT: usize = 25;

fn main() {
    run_gol();
}

fn run_gol() {
    terminal().clear(ClearType::All).unwrap();

    let cursor = cursor();

    print_info("Running Game-Of-Life reference implementation...\n");

    cursor.save_position();

    let mut gol = random();

    for i in 0.. {
        println!("width={} height={} {}{}ms", WIDTH, HEIGHT, "\u{2764}", UPDATE_INTERVAL);
        print!("{}", gol);
        std::thread::sleep(std::time::Duration::from_millis(UPDATE_INTERVAL));
        let start = Instant::now();
        gol.next_gen();
        let stop = start.elapsed();
        println!("#{} {}{}s:{}ns", i, "\u{23f1}", stop.as_secs(), stop.subsec_nanos());
        cursor.reset_position();
    }
}

pub fn print_info(info: &str) {
    println!("{}", style(info.to_string()).with(Color::Yellow));
}

fn carlos() -> Universe {
    unimplemented!()
}

fn random() -> Universe {
    let mut rng = rand::thread_rng();

    let mut gol = Universe::new(WIDTH, HEIGHT);

    for i in 0..WIDTH * HEIGHT {
        let alive = rng.gen::<bool>();
        let (x, y) = gol.get_position(i);
        if alive {
            gol.set_alive(x, y)
        }
    }

    gol
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

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Cell {
    Dead = 0,
    Alive = 1,
}

struct Universe {
    pub width: usize,
    pub height: usize,
    cells: Vec<Cell>,
}

impl Universe {
    pub fn new(width: usize, height: usize) -> Self {
        Universe { width, height, cells: vec![Cell::Dead; width * height] }
    }

    pub fn set_alive(&mut self, x: usize, y: usize) {
        let index = self.get_index(x, y);
        self.cells[index] = Cell::Alive;
    }

    pub fn next_gen(&mut self) {
        let mut cells = vec![Cell::Dead; self.cells.len()];

        for (i, cell) in self.cells.iter().enumerate() {
            let (x, y) = self.get_position(i);
            let num_neighbors = self.num_alive_neighbors(x, y);

            match cell {
                Cell::Alive if num_neighbors < 2 || num_neighbors > 3 => {
                    cells[i] = Cell::Dead;
                }
                Cell::Dead if num_neighbors == 3 => {
                    cells[i] = Cell::Alive;
                }
                _ => cells[i] = *cell,
            }
            /*
            println!(
                "{:<3} ({},{}): state={:?} nb={} ==> new_state={:?}",
                i, x, y, cell, num_neighbors, cells[i]
            );
            */
        }

        self.cells = cells;
    }

    fn num_alive_neighbors(&self, x: usize, y: usize) -> usize {
        let mut num_alive = 0;
        let mut dir = Direction::first();

        loop {
            let (nb_x, nb_y) = self.get_neighbor_position(x, y, &dir);
            let index = self.get_index(nb_x, nb_y);

            if self.cells[index] == Cell::Alive {
                num_alive += 1;
            }

            if let Some(next_dir) = dir.next() {
                dir = next_dir;
            } else {
                break;
            }
        }

        num_alive
    }

    fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn get_neighbor_position(
        &self,
        x: usize,
        y: usize,
        dir: &Direction,
    ) -> (usize, usize) {
        match dir {
            Direction::North => match (x, y) {
                (_, y) if y == 0 => (x, self.height - 1),
                (_, _) => (x, y - 1),
            },
            Direction::NorthEast => match (x, y) {
                (x, 0) if x == self.width - 1 => (0, self.height - 1),
                (x, _) if x == self.width - 1 => (0, y - 1),
                (_, 0) => (x + 1, self.height - 1),
                (_, _) => (x + 1, y - 1),
            },
            Direction::East => match (x, y) {
                (x, _) if x == self.width - 1 => (0, y),
                (_, _) => (x + 1, y),
            },
            Direction::SouthEast => match (x, y) {
                (x, y) if x == self.width - 1 && y == self.height - 1 => (0, 0),
                (x, _) if x == self.width - 1 => (0, y + 1),
                (_, y) if y == self.height - 1 => (x + 1, 0),
                (_, _) => (x + 1, y + 1),
            },
            Direction::South => match (x, y) {
                (_, y) if y == self.height - 1 => (x, 0),
                (_, _) => (x, y + 1),
            },
            Direction::SouthWest => match (x, y) {
                (0, y) if y == self.height - 1 => (self.width - 1, 0),
                (0, _) => (self.width - 1, y + 1),
                (_, y) if y == self.height - 1 => (x - 1, 0),
                (_, _) => (x - 1, y + 1),
            },
            Direction::West => match (x, y) {
                (0, _) => (self.width - 1, y),
                (_, _) => (x - 1, y),
            },
            Direction::NorthWest => match (x, y) {
                (0, 0) => (self.width - 1, self.height - 1),
                (0, _) => (self.width - 1, y - 1),
                (_, 0) => (x - 1, self.height - 1),
                (_, _) => (x - 1, y - 1),
            },
        }
    }

    fn get_position(&self, index: usize) -> (usize, usize) {
        (index % self.width, index / self.width)
    }

    fn get_x_from_index(&self, index: usize) -> usize {
        index % self.width
    }
}

impl std::fmt::Display for Universe {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (i, cell) in self.cells.iter().enumerate() {
            let x = self.get_x_from_index(i);

            match cell {
                Cell::Alive if x < self.width - 1 => {
                    write!(f, "◼")?;
                }
                Cell::Alive if x == self.width - 1 => {
                    writeln!(f, "◼")?;
                }
                Cell::Dead if x < self.width - 1 => {
                    write!(f, "◻")?;
                }
                Cell::Dead if x == self.width - 1 => {
                    writeln!(f, "◻")?;
                }
                _ => (),
            }
        }

        writeln!(f, "")
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

impl Direction {
    pub fn first() -> Self {
        Direction::North
    }
}

impl Iterator for Direction {
    type Item = Direction;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Direction::North => Some(Direction::NorthEast),
            Direction::NorthEast => Some(Direction::East),
            Direction::East => Some(Direction::SouthEast),
            Direction::SouthEast => Some(Direction::South),
            Direction::South => Some(Direction::SouthWest),
            Direction::SouthWest => Some(Direction::West),
            Direction::West => Some(Direction::NorthWest),
            Direction::NorthWest => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_alive() {
        let mut u = Universe::new(4, 4);

        assert_eq!(Cell::Dead, u.cells[10]);

        u.set_alive(2, 2);

        assert_eq!(Cell::Alive, u.cells[10]);
    }

    #[test]
    fn simple_universe() {
        let mut u = Universe::new(5, 5);

        u.set_alive(2, 1);
        u.set_alive(2, 2);
        u.set_alive(2, 3);

        assert_eq!(1, u.num_alive_neighbors(1, 0));
        assert_eq!(1, u.num_alive_neighbors(2, 0));
        assert_eq!(1, u.num_alive_neighbors(3, 0));
        assert_eq!(2, u.num_alive_neighbors(1, 1));
        assert_eq!(1, u.num_alive_neighbors(2, 1));
        assert_eq!(2, u.num_alive_neighbors(3, 1));
        assert_eq!(3, u.num_alive_neighbors(1, 2));
        assert_eq!(2, u.num_alive_neighbors(2, 2));
        assert_eq!(3, u.num_alive_neighbors(3, 2));
        assert_eq!(2, u.num_alive_neighbors(1, 3));
        assert_eq!(1, u.num_alive_neighbors(2, 3));
        assert_eq!(2, u.num_alive_neighbors(3, 3));
        assert_eq!(1, u.num_alive_neighbors(1, 4));
        assert_eq!(1, u.num_alive_neighbors(2, 4));
        assert_eq!(1, u.num_alive_neighbors(3, 4));
    }

    #[test]
    fn num_neighbors_alive_within_universe() {
        let mut u = Universe::new(4, 4);
        u.set_alive(2, 2);

        assert_eq!(1, u.num_alive_neighbors(1, 1));
        assert_eq!(1, u.num_alive_neighbors(2, 1));
        assert_eq!(1, u.num_alive_neighbors(3, 1));
        assert_eq!(1, u.num_alive_neighbors(3, 2));
        assert_eq!(1, u.num_alive_neighbors(3, 3));
        assert_eq!(1, u.num_alive_neighbors(2, 3));
        assert_eq!(1, u.num_alive_neighbors(1, 3));
        assert_eq!(1, u.num_alive_neighbors(1, 2));
    }

    #[test]
    fn num_neighbors_alive_top_left() {
        let mut u = Universe::new(4, 4);
        u.set_alive(0, 0);

        assert_eq!(1, u.num_alive_neighbors(0, 3));
        assert_eq!(1, u.num_alive_neighbors(1, 3));
        assert_eq!(1, u.num_alive_neighbors(1, 0));
        assert_eq!(1, u.num_alive_neighbors(1, 1));
        assert_eq!(1, u.num_alive_neighbors(0, 1));
        assert_eq!(1, u.num_alive_neighbors(3, 1));
        assert_eq!(1, u.num_alive_neighbors(3, 0));
        assert_eq!(1, u.num_alive_neighbors(3, 3));
    }

    #[test]
    fn num_neighbors_alive_top_right() {
        let mut u = Universe::new(4, 4);
        u.set_alive(3, 0);

        assert_eq!(1, u.num_alive_neighbors(3, 3));
        assert_eq!(1, u.num_alive_neighbors(0, 3));
        assert_eq!(1, u.num_alive_neighbors(0, 0));
        assert_eq!(1, u.num_alive_neighbors(0, 1));
        assert_eq!(1, u.num_alive_neighbors(3, 1));
        assert_eq!(1, u.num_alive_neighbors(2, 1));
        assert_eq!(1, u.num_alive_neighbors(2, 0));
        assert_eq!(1, u.num_alive_neighbors(2, 3));
    }

    #[test]
    fn num_neighbors_alive_bottom_right() {
        let mut u = Universe::new(4, 4);
        u.set_alive(3, 3);

        assert_eq!(1, u.num_alive_neighbors(3, 2));
        assert_eq!(1, u.num_alive_neighbors(0, 2));
        assert_eq!(1, u.num_alive_neighbors(0, 3));
        assert_eq!(1, u.num_alive_neighbors(0, 0));
        assert_eq!(1, u.num_alive_neighbors(3, 0));
        assert_eq!(1, u.num_alive_neighbors(2, 0));
        assert_eq!(1, u.num_alive_neighbors(2, 3));
        assert_eq!(1, u.num_alive_neighbors(2, 2));
    }

    #[test]
    fn num_neighbors_alive_bottom_left() {
        let mut u = Universe::new(4, 4);
        u.set_alive(0, 3);

        assert_eq!(1, u.num_alive_neighbors(0, 2));
        assert_eq!(1, u.num_alive_neighbors(1, 2));
        assert_eq!(1, u.num_alive_neighbors(1, 3));
        assert_eq!(1, u.num_alive_neighbors(1, 0));
        assert_eq!(1, u.num_alive_neighbors(0, 0));
        assert_eq!(1, u.num_alive_neighbors(3, 0));
        assert_eq!(1, u.num_alive_neighbors(3, 3));
        assert_eq!(1, u.num_alive_neighbors(3, 2));
    }
}
