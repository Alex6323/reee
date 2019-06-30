#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

pub struct Universe {
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

    pub fn get_position(&self, index: usize) -> (usize, usize) {
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
                    write!(f, "■")?;
                }
                Cell::Alive if x == self.width - 1 => {
                    writeln!(f, "■")?;
                }
                Cell::Dead if x < self.width - 1 => {
                    //write!(f, "□")?;
                    write!(f, " ")?;
                }
                Cell::Dead if x == self.width - 1 => {
                    //writeln!(f, "□")?;
                    writeln!(f, " ")?;
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
