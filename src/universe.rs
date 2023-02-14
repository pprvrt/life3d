use std::fmt;
use rand::Rng;

#[derive(Clone, Copy)]
pub enum Cell {
    Dead,
    Alive
}

pub struct Universe {
    pub width: u32,
    pub height: u32,
    pub cells: Vec<Cell>
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.cells.chunks(self.width as usize) {
            for &cell in line {
                match cell {
                    Cell::Alive => write!(f, "◼")?,
                    Cell::Dead => write!(f, " ")?
                }
            }
            writeln!(f)?
        }
        Ok(())
    }
}

impl Universe {
    fn index(&self, cx: u32, cy: u32) -> usize {
        (cy * self.width + cx) as usize
    }

    pub fn alive(&self, index: usize) -> bool {
        if let Cell::Alive = self.cells[index] {
            return true
        }
        false
    }

    pub fn step(&mut self) {
        let mut next = self.cells.clone();
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.index(x, y);
                let actual = self.cells[idx];

                // https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life
                let cell = match (actual, self.neighbours(x, y)) {
                    (Cell::Alive, n) if n < 2 => Cell::Dead,
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    (Cell::Alive, n) if n > 3 => Cell::Dead,
                    (Cell::Dead, 3) => Cell::Alive,
                    (dontchange, _) => dontchange
                };
                next[idx] = cell;
            }
        }
        self.cells = next
    }

    fn neighbours(&self, x: u32, y: u32) -> u8 {
        let mut count: u8 = 0;
        for nx in [self.width-1, 0, 1] {
            for ny in [self.height-1, 0, 1] {
                if nx == 0 && ny == 0 { continue }
                let cx = (x + nx) % self.width;
                let cy = (y + ny) % self.height;
                count += self.cells[self.index(cx, cy)] as u8;
            }
        }
        return count
    }

    pub fn rand(&mut self) {
        let mut rng = rand::thread_rng();
        let cells = (0..self.width*self.height).map(
            |_| {
                if rng.gen_bool(0.5) { Cell::Alive }
                else { Cell::Dead }
            }
        ).collect();
        self.cells = cells;
    }

    pub fn new(width: u32, height: u32) -> Universe {
        Universe {
            width,
            height,
            cells: vec![Cell::Dead;(width*height) as usize]
        }
    }
}