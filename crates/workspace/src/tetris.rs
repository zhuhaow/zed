use gpui::{div, IntoElement, ParentElement, RenderOnce, Styled, WindowContext};

const GRID_WIDTH: usize = 10;
const GRID_HEIGHT: usize = 20;

#[derive(Debug, Clone, Copy)]
enum Cell {
    Empty,
    Occupied,
}

type Rotation = [(isize, isize); 4];

#[derive(Debug, Clone, Copy)]
pub struct Tetromino {
    shape: Shape,
    rotations: [Rotation; 4],
    current_rotation: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum Shape {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl Tetromino {
    pub fn new(shape: Shape) -> Self {
        let rotations = Self::default_rotation(&shape);
        Self {
            shape,
            rotations,
            current_rotation: 0,
        }
    }

    pub fn random() -> Self {
        use rand::Rng;
        let shape = match rand::thread_rng().gen_range(0..=6) {
            0 => Shape::I,
            1 => Shape::O,
            2 => Shape::T,
            3 => Shape::S,
            4 => Shape::Z,
            5 => Shape::J,
            6 => Shape::L,
            _ => unreachable!(),
        };
        Self::new(shape)
    }

    fn default_rotation(shape: &Shape) -> [Rotation; 4] {
        match shape {
            Shape::I => [
                [(0, 0), (1, 0), (2, 0), (3, 0)],
                [(0, 0), (0, 1), (0, 2), (0, 3)],
                [(0, 0), (1, 0), (2, 0), (3, 0)],
                [(0, 0), (0, 1), (0, 2), (0, 3)],
            ],
            // Remaining shapes...
            _ => unimplemented!(),
        }
    }

    pub fn rotate(&mut self) {
        self.current_rotation = (self.current_rotation + 1) % 4;
    }

    pub fn current_rotation(&self) -> &Rotation {
        &self.rotations[self.current_rotation]
    }
}

#[derive(IntoElement)]
pub struct Tetris {
    score: usize,
    grid: [[Cell; GRID_WIDTH]; GRID_HEIGHT],
    next_tetromino: Option<Tetromino>,
    current_tetromino: Option<Tetromino>,
    current_position: (isize, isize),
    history: Vec<Tetromino>,
}

impl Tetris {
    pub fn new() -> Self {
        Self {
            score: 0,
            grid: [[Cell::Empty; GRID_WIDTH]; GRID_HEIGHT],
            next_tetromino: None,
            current_tetromino: None,
            current_position: (0, 0),
            history: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn update(&mut self) {
        if self.current_tetromino.is_none() {
            self.spawn_tetromino(GRID_WIDTH / 2);
            self.next_tetromino = Some(Tetromino::random());
        } else {
            if !self.move_piece_down() {
                self.fix_piece();
                let complete_rows = &self.check_rows();
                // Clear rows and calculate score
                for row in complete_rows {
                    self.clear_and_shift_row(*row);
                    self.score += Self::points_for_rows(complete_rows.len());
                }
            }

            if self.is_game_over() {
                println!("Game over, final score: {}", self.score);
                self.reset();
            }
        }
    }

    pub fn spawn_tetromino(&mut self, column: usize) {
        let tetromino = self
            .next_tetromino
            .take()
            .unwrap_or_else(|| Tetromino::random());
        // Positions the Tetrimino horizontally based on the given column,
        // and vertically just off the top of the grid
        self.current_position = (column as isize, -1);
        self.current_tetromino = Some(tetromino);
    }

    pub fn move_piece_down(&mut self) -> bool {
        if let Some(tetromino) = self.current_tetromino {
            let new_position = (self.current_position.0, self.current_position.1 + 1);
            if self.validate_position(tetromino, new_position) {
                self.current_position = new_position;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn validate_position(&self, tetromino: Tetromino, position: (isize, isize)) -> bool {
        for &(x, y) in tetromino.current_rotation() {
            let cell_x = position.0 + x;
            let cell_y = position.1 + y;

            // If cell is off the grid, or if it's occupied, the position is invalid
            if cell_y >= GRID_HEIGHT as isize
                || cell_x < 0
                || cell_x >= GRID_WIDTH as isize
                || (cell_y >= 0
                    && matches!(self.grid[cell_y as usize][cell_x as usize], Cell::Occupied))
            {
                return false;
            }
        }
        true
    }

    pub fn fix_piece(&mut self) {
        if let Some(tetromino) = self.current_tetromino {
            for &(x, y) in tetromino.current_rotation() {
                let cell_x = self.current_position.0 + x;
                let cell_y = self.current_position.1 + y;

                if cell_y >= 0 {
                    self.grid[cell_y as usize][cell_x as usize] = Cell::Occupied;
                }
            }

            self.current_tetromino = None;
        }
    }

    pub fn check_rows(&mut self) -> Vec<usize> {
        let mut rows_to_clear = Vec::new();
        for (row_index, row) in self.grid.iter().enumerate() {
            let mut row_is_full = true;
            for cell in row.iter() {
                if let Cell::Empty = cell {
                    row_is_full = false;
                    break;
                }
            }
            if row_is_full {
                rows_to_clear.push(row_index);
            }
        }
        rows_to_clear
    }

    pub fn points_for_rows(rows: usize) -> usize {
        match rows {
            1 => 100, // Single
            2 => 300, // Double
            3 => 500, // Triple
            4 => 800, // Tetris
            _ => 0,
        }
    }

    pub fn next_frame(&mut self) {
        let complete_rows = self.check_rows();
        if !complete_rows.is_empty() {
            self.score += Self::points_for_rows(complete_rows.len());
            for row in complete_rows {
                self.clear_and_shift_row(row);
            }
        }
    }

    pub fn update_row(&mut self, row: usize, new_row: [Cell; GRID_WIDTH]) {
        self.grid[row] = new_row;
    }

    pub fn shift_row_down(&mut self, row: usize) {
        if row > 0 {
            self.grid[row] = self.grid[row - 1];
        }
    }

    pub fn clear_row(&mut self, row: usize) {
        self.update_row(row, [Cell::Empty; GRID_WIDTH]);
    }

    pub fn clear_and_shift_row(&mut self, row: usize) {
        self.clear_row(row);
        for index in (1..=row).rev() {
            self.shift_row_down(index);
        }
    }

    pub fn is_game_over(&self) -> bool {
        // If there is an occupied cell on the top row of the grid, the game is over
        self.grid[0]
            .iter()
            .any(|&cell| matches!(cell, Cell::Occupied))
    }
}

impl RenderOnce for Tetris {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        div()
            .size_full()
            .items_center()
            .flex()
            .flex_col()
            .gap_px()
            .children(
                self.grid
                    .iter()
                    .map(|row| {
                        div().flex().gap_px().children(
                            row.iter()
                                .map(|cell| match cell {
                                    Cell::Empty => div().w_8().h_8().bg(gpui::red()),
                                    Cell::Occupied => div().w_8().h_8().bg(gpui::blue()),
                                })
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
    }
}
