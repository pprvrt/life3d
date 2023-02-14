use rand::Rng;
use std::fmt;

#[derive(Clone, Copy, PartialEq)]
pub enum CellState {
    Dead,
    Alive,
}

#[derive(Clone, Copy)]
pub struct Cell {
    pub state: CellState,
    pub changed: bool,
}

pub struct Universe {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.cells.chunks(self.width as usize) {
            for &cell in line {
                match cell.state {
                    CellState::Alive => write!(f, "◼")?,
                    CellState::Dead => write!(f, " ")?,
                }
            }
            writeln!(f)?
        }
        Ok(())
    }
}

impl Universe {
    fn index(&self, cx: usize, cy: usize) -> usize {
        (cy * self.width + cx) as usize
    }

    pub fn is_alive(&self, index: usize) -> bool {
        if let CellState::Alive = self.cells[index].state {
            return true;
        }
        false
    }

    pub fn has_changed(&self, index: usize) -> bool {
        self.cells[index].changed
    }

    pub fn step(&mut self) {
        let mut next = self.cells.clone();
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.index(x, y);
                let actual = self.cells[idx];

                // https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life
                let cellstate = match (actual.state, self.neighbours(x, y)) {
                    (CellState::Alive, n) if n < 2 => CellState::Dead,
                    (CellState::Alive, 2) | (CellState::Alive, 3) => CellState::Alive,
                    (CellState::Alive, n) if n > 3 => CellState::Dead,
                    (CellState::Dead, 3) => CellState::Alive,
                    (dontchange, _) => dontchange,
                };

                next[idx] = Cell {
                    state: cellstate,
                    changed: actual.state != cellstate,
                };
            }
        }
        self.cells = next
    }

    fn neighbours(&self, x: usize, y: usize) -> u8 {
        let mut count: u8 = 0;
        for nx in [self.width - 1, 0, 1] {
            for ny in [self.height - 1, 0, 1] {
                if nx == 0 && ny == 0 {
                    continue;
                }
                let cx = (x + nx) % self.width;
                let cy = (y + ny) % self.height;
                count += self.cells[self.index(cx, cy)].state as u8;
            }
        }
        count
    }

    pub fn rand(&mut self) {
        let mut rng = rand::thread_rng();
        let mut cells: Vec<Cell> = Vec::new();

        for (_, cell) in (0..self.width * self.height).zip(self.cells.iter_mut()) {
            let is_alive = rng.gen_bool(0.5);
            let state = if is_alive {
                CellState::Alive
            } else {
                CellState::Dead
            };
            cells.push(Cell {
                state,
                changed: cell.state != state
            })
        }
        self.cells = cells;
    }

    pub fn one(&mut self) {
        self.cells[(self.height/2)*self.width + self.width/2].state = CellState::Alive;
        self.cells[(self.height/2)*self.width + self.width/2].changed = true;
    }

    pub fn new(width: usize, height: usize) -> Universe {
        Universe {
            width,
            height,
            cells: vec![
                Cell {
                    state: CellState::Dead,
                    changed: true
                };
                (width * height) as usize
            ],
        }
    }
}
