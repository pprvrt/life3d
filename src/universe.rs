use rand::Rng;
use std::fmt;

#[derive(Clone, Copy, PartialEq)]
pub enum CellState {
    Dead,
    Alive,
}

#[derive(Clone, Copy)]
pub struct Cell {
    state: CellState,
    changed: bool,
}

pub struct Universe {
    width: usize,
    height: usize,
    cells: Vec<Cell>
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.cells.chunks(self.width) {
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
    pub fn index(&self, cx: usize, cy: usize) -> usize {
        cy * self.width + cx
    }

    pub fn size(&self) -> usize {
        self.width * self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
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

    pub fn toggle(&mut self, x: usize, y: usize) {
        let index = self.index(x, y);
        self.cells[index].state = match self.cells[index].state {
            CellState::Dead => CellState::Alive,
            CellState::Alive => CellState::Dead
        };
        self.cells[index].changed = true;
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
                changed: cell.state != state,
            })
        }
        self.cells = cells;
    }

    pub fn clear(&mut self) {
        let mut cells: Vec<Cell> = Vec::new();

        for (_, cell) in (0..self.width * self.height).zip(self.cells.iter_mut()) {
            cells.push(Cell {
                state: CellState::Dead,
                changed: cell.state == CellState::Alive,
            })
        }
        self.cells = cells;
    }

    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
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
                width * height
            ],
        }
    }
}
