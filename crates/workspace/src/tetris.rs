#[allow(unused)]
use gpui::{div, IntoElement, ParentElement, RenderOnce, Styled, WindowContext};
use serde::{Deserialize, Serialize};

const GRID_WIDTH: usize = 10;
const GRID_HEIGHT: usize = 20;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Block {
    pub status: BlockStatus,
}

pub type Rotation = [(isize, isize); 4];
pub type Rotations = Vec<Rotation>;

#[derive(Clone, Copy)]
pub enum Shape {
    I,
    // O,
    T,
    // S,
    // Z,
    // J,
    // L,
}

impl From<Shape> for Rotations {
    fn from(shape: Shape) -> Self {
        let blocks = match shape {
            Shape::I => {
                vec![
                    [(0, 0), (-1, 0), (1, 0), (2, 0)], // Up/Down
                    [(0, 0), (0, -1), (0, 1), (0, 2)], // Right/Left
                    [(0, 0), (-1, 0), (1, 0), (2, 0)], // Up/Down
                    [(0, 0), (0, -1), (0, 1), (0, 2)], // Right/Left
                ]
            }
            Shape::T => {
                vec![
                    [(0, 0), (-1, 0), (1, 0), (0, 1)],  // Up
                    [(0, 0), (0, -1), (0, 1), (-1, 0)], // Right
                    [(0, 0), (-1, 0), (1, 0), (0, -1)], // Down
                    [(0, 0), (0, -1), (0, 1), (1, 0)],  // Left
                ]
            }
        };

        blocks
    }
}

#[derive(Clone, Copy)]
pub enum BlockStatus {
    Empty,
    Occupied,
}

pub struct Tetromino {
    rotations: Rotations,
    shape: Shape,
    rotation: usize,
}

impl Tetromino {
    pub fn new(shape: Shape) -> Self {
        let rotations = shape.clone().into();
        Self {
            shape,
            rotation: 0,
            rotations,
        }
    }
    pub fn rotate(&mut self) {
        self.rotation = (self.rotation + 1) % 4;
    }

    pub fn rotations(&self) -> &Rotations {
        &self.rotations
    }

    pub fn current_rotation(&self) -> &Rotation {
        &self.rotations[self.rotation]
    }
}

#[derive(Serialize, Deserialize)]
pub struct Grid {
    grid_blocks: [[BlockStatus; GRID_WIDTH]; GRID_HEIGHT],
}

impl Grid {
    /// Creates a new grid where every cell is empty.
    /// The dimensions of the grid are defined by `GRID_WIDTH` and `GRID_HEIGHT`.
    ///
    /// # Returns
    /// A new instance of `Grid`
    pub fn new() -> Self {
        Self {
            grid_blocks: [[BlockStatus::Empty; GRID_WIDTH]; GRID_HEIGHT],
        }
    }

    /// Checks if the tetromino can be placed at the given position on the grid.
    /// Iterates over each block in the current rotation of the tetromino, translating
    /// its relative position to absolute grid coordinates.
    ///
    /// # Parameters
    /// - `tetromino`: The tetromino to place on the grid
    /// - `position`: A tuple (isize, isize) representing the position on the grid
    ///               of the tetromino's origin point
    ///
    /// # Returns
    /// `true` if the tetromino can be placed on the grid at the given position,
    /// `false` otherwise.
    pub fn place_piece(&self, tetromino: &Tetromino, position: (isize, isize)) -> bool {
        for &(x, y) in tetromino.current_rotation() {
            let cell_x = position.0 + x;
            let cell_y = position.1 + y;

            // If cell is off the grid, or if it's occupied, the position is invalid
            if cell_y < 0
                || cell_y >= GRID_HEIGHT as isize
                || cell_x < 0
                || cell_x >= GRID_WIDTH as isize
                || matches!(
                    self.grid_blocks[cell_y as usize][cell_x as usize],
                    BlockStatus::Occupied
                )
            {
                return false;
            }
        }
        true
    }

    /// Fixes the tetromino to the grid, marking its blocks as occupied.
    /// This should be called when the tetromino reaches a position where it
    /// can't move down any further. All blocks that are part of the tetromino
    /// in its current rotation at the current position will be marked as occupied
    /// in the grid.
    ///
    /// # Parameters
    /// - `tetromino`: The tetromino to fix on the grid
    /// - `position`: A tuple (isize, isize) representing the position on the grid
    ///               of the tetromino's origin point
    pub fn fix_piece(&mut self, tetromino: &Tetromino, position: (isize, isize)) {
        for &(x, y) in tetromino.current_rotation() {
            let cell_x = position.0 + x;
            let cell_y = position.1 + y;

            // As the piece has already been moved, no need to validate
            // just update the block status
            if cell_y >= 0 {
                self.grid_blocks[cell_y as usize][cell_x as usize] = BlockStatus::Occupied;
            }
        }
    }

    /// Checks each row in the grid to see if it is full (i.e., completed).
    ///
    /// # Returns
    /// A vector of row indices that are full and should be cleared.
    pub fn check_complete_rows(&self) -> Vec<usize> {
        let mut complete_rows = Vec::new();

        for (i, row) in self.grid_blocks.iter().enumerate() {
            if row
                .iter()
                .all(|&status| matches!(status, BlockStatus::Occupied))
            {
                complete_rows.push(i);
            }
        }

        complete_rows
    }

    /// Clears completed rows from the grid and shifts down the rows above them.
    ///
    /// # Parameters
    /// - `rows`: The vector of row indices to be cleared.
    pub fn clear_and_shift_rows(&mut self, rows: Vec<usize>) {
        // The rows are going to be coming in sorted from low to high indices,
        // which is necessary to avoid dynamic index changes after row removal.
        for &row in rows.iter().rev() {
            // reverse to handle from bottom to top
            for current_row in (1..=row).rev() {
                self.grid_blocks[current_row] = self.grid_blocks[current_row - 1];
            }

            // The top row is newly empty
            self.grid_blocks[0] = [BlockStatus::Empty; GRID_WIDTH];
        }
    }

    /// Serializes the grid into a JSON string.
    ///
    /// # Returns
    /// `Ok(String)` containing the serialized JSON string of the grid if successful,
    /// or an error if serialization fails.
    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializes a JSON string into a Grid instance.
    ///
    /// # Parameters
    /// - `data`: The JSON string representation of a grid.
    ///
    /// # Returns
    /// `Ok(Grid)` if the deserialization is successful, or an error if it fails.
    pub fn deserialize(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }

    /// Saves the grid state to a file.
    ///
    /// # Parameters
    /// - `filename`: The path to the file where the grid state will be saved.
    ///
    /// # Returns
    /// `Ok(())` if the grid state is successfully written to the file, or an error if it fails.
    pub fn save_to_file(&self, filename: &str) -> Result<(), io::Error> {
        let serialized = self.serialize()?;
        fs::write(filename, serialized)?;
        Ok(())
    }

    /// Loads the grid state from a file.
    ///
    /// # Parameters
    /// - `filename`: The path to the file from which to load the grid state.
    ///
    /// # Returns
    /// `Ok(Grid)` if the grid state is successfully loaded from the file, or an error if it fails.
    pub fn load_from_file(filename: &str) -> Result<Self, io::Error> {
        let mut file = fs::File::open(filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let grid = Grid::deserialize(&contents)?;
        Ok(grid)
    }
}

// impl Grid {
//     pub fn new() -> Self {
//         // Initialize the grid
//     }

//     pub fn place_tetromino(&mut self, tetromino: &Tetromino, position: (usize, usize)) {
//         // Place a tetromino in the grid
//     }

//     pub fn check_rows(&mut self) -> Vec<usize> {
//         // Check if full rows need to be removed and return their indices
//     }

//     pub fn remove_row(&mut self, row: usize) {
//         // Remove a row from the grid and shift everything down
//     }
// }

// pub struct Player {
//     // Information about the player, e.g., current score
// }

// pub struct Score {
//     points: usize,
// }

// impl Score {
//     pub fn new() -> Self {
//         // Initialize the score
//     }

//     pub fn increase_score(&mut self, points: usize) {
//         // Increase the score
//     }
// }

// pub struct Game {
//     grid: Grid,
//     player: Player,
//     score: Score,
//     current_tetromino: Tetromino,
// }

// impl Game {
//     pub fn new() -> Self {
//         // Init the game
//     }

//     pub fn update(&mut self) {
//         // Run one iteration of the game logic
//     }
// }

// pub struct Utils;

// impl Utils {
//     pub fn get_random_tetromino() -> Tetromino {
//         // Return a random tetromino
//     }
// }

// -----------------------------------------------------------------------------

// use gpui::{div, IntoElement, ParentElement, RenderOnce, Styled, WindowContext};

// const GRID_WIDTH: usize = 10;
// const GRID_HEIGHT: usize = 20;

// #[derive(Debug, Clone, Copy)]
// enum Cell {
//     Empty,
//     Occupied,
// }

// type Rotation = [(isize, isize); 4];

// #[derive(Debug, Clone, Copy)]
// pub struct Tetromino {
//     shape: Shape,
//     rotations: [Rotation; 4],
//     current_rotation: usize,
// }

// #[derive(Debug, Clone, Copy)]
// pub enum Shape {
//     I,
//     O,
//     T,
//     S,
//     Z,
//     J,
//     L,
// }

// impl Tetromino {
//     pub fn new(shape: Shape) -> Self {
//         let rotations = Self::default_rotation(&shape);
//         Self {
//             shape,
//             rotations,
//             current_rotation: 0,
//         }
//     }

//     pub fn random() -> Self {
//         use rand::Rng;
//         let shape = match rand::thread_rng().gen_range(0..=6) {
//             0 => Shape::I,
//             1 => Shape::O,
//             2 => Shape::T,
//             3 => Shape::S,
//             4 => Shape::Z,
//             5 => Shape::J,
//             6 => Shape::L,
//             _ => unreachable!(),
//         };
//         Self::new(shape)
//     }

//     fn default_rotation(shape: &Shape) -> [Rotation; 4] {
//         match shape {
//             Shape::I => [
//                 [(0, 0), (1, 0), (2, 0), (3, 0)],
//                 [(0, 0), (0, 1), (0, 2), (0, 3)],
//                 [(0, 0), (1, 0), (2, 0), (3, 0)],
//                 [(0, 0), (0, 1), (0, 2), (0, 3)],
//             ],
//             // Remaining shapes...
//             _ => unimplemented!(),
//         }
//     }

//     pub fn rotate(&mut self) {
//         self.current_rotation = (self.current_rotation + 1) % 4;
//     }

//     pub fn current_rotation(&self) -> &Rotation {
//         &self.rotations[self.current_rotation]
//     }
// }

// #[derive(IntoElement)]
// pub struct Tetris {
//     score: usize,
//     grid: [[Cell; GRID_WIDTH]; GRID_HEIGHT],
//     next_tetromino: Option<Tetromino>,
//     current_tetromino: Option<Tetromino>,
//     current_position: (isize, isize),
//     history: Vec<Tetromino>,
// }

// impl Tetris {
//     pub fn new() -> Self {
//         Self {
//             score: 0,
//             grid: [[Cell::Empty; GRID_WIDTH]; GRID_HEIGHT],
//             next_tetromino: None,
//             current_tetromino: None,
//             current_position: (0, 0),
//             history: Vec::new(),
//         }
//     }

//     pub fn reset(&mut self) {
//         *self = Self::new();
//     }

//     pub fn update(&mut self) {
//         if self.current_tetromino.is_none() {
//             self.spawn_piece(GRID_WIDTH / 2);
//             self.next_tetromino = Some(Tetromino::random());
//         } else {
//             if !self.move_piece_down() {
//                 self.fix_piece();
//                 let complete_rows = &self.check_rows();
//                 // Clear rows and calculate score
//                 for row in complete_rows {
//                     self.clear_and_shift_row(*row);
//                     self.score += Self::points_for_rows(complete_rows.len());
//                 }
//             }

//             if self.is_game_over() {
//                 println!("Game over, final score: {}", self.score);
//                 self.reset();
//             }
//         }
//     }

//     pub fn spawn_piece(&mut self, column: usize) {
//         let tetromino = self
//             .next_tetromino
//             .take()
//             .unwrap_or_else(|| Tetromino::random());
//         // Positions the Tetrimino horizontally based on the given column,
//         // and vertically just off the top of the grid
//         self.current_position = (column as isize, -1);
//         self.current_tetromino = Some(tetromino);
//     }

//     pub fn fix_piece(&mut self) {
//         if let Some(tetromino) = self.current_tetromino {
//             for &(x, y) in tetromino.current_rotation() {
//                 let cell_x = self.current_position.0 + x;
//                 let cell_y = self.current_position.1 + y;

//                 if cell_y >= 0 {
//                     self.grid[cell_y as usize][cell_x as usize] = Cell::Occupied;
//                 }
//             }

//             self.current_tetromino = None;
//         }
//     }

//     pub fn move_piece_down(&mut self) -> bool {
//         if let Some(tetromino) = self.current_tetromino {
//             let new_position = (self.current_position.0, self.current_position.1 + 1);
//             if self.validate_position(tetromino, new_position) {
//                 self.current_position = new_position;
//                 true
//             } else {
//                 false
//             }
//         } else {
//             false
//         }
//     }

//     fn validate_position(&self, tetromino: Tetromino, position: (isize, isize)) -> bool {
//         for &(x, y) in tetromino.current_rotation() {
//             let cell_x = position.0 + x;
//             let cell_y = position.1 + y;

//             // If cell is off the grid, or if it's occupied, the position is invalid
//             if cell_y >= GRID_HEIGHT as isize
//                 || cell_x < 0
//                 || cell_x >= GRID_WIDTH as isize
//                 || (cell_y >= 0
//                     && matches!(self.grid[cell_y as usize][cell_x as usize], Cell::Occupied))
//             {
//                 return false;
//             }
//         }
//         true
//     }

//     pub fn check_rows(&mut self) -> Vec<usize> {
//         let mut rows_to_clear = Vec::new();
//         for (row_index, row) in self.grid.iter().enumerate() {
//             let mut row_is_full = true;
//             for cell in row.iter() {
//                 if let Cell::Empty = cell {
//                     row_is_full = false;
//                     break;
//                 }
//             }
//             if row_is_full {
//                 rows_to_clear.push(row_index);
//             }
//         }
//         rows_to_clear
//     }

//     pub fn points_for_rows(rows: usize) -> usize {
//         match rows {
//             1 => 100, // Single
//             2 => 300, // Double
//             3 => 500, // Triple
//             4 => 800, // Tetris
//             _ => 0,
//         }
//     }

//     pub fn next_frame(&mut self) {
//         let complete_rows = self.check_rows();
//         if !complete_rows.is_empty() {
//             self.score += Self::points_for_rows(complete_rows.len());
//             for row in complete_rows {
//                 self.clear_and_shift_row(row);
//             }
//         }
//     }

//     pub fn update_row(&mut self, row: usize, new_row: [Cell; GRID_WIDTH]) {
//         self.grid[row] = new_row;
//     }

//     pub fn shift_row_down(&mut self, row: usize) {
//         if row > 0 {
//             self.grid[row] = self.grid[row - 1];
//         }
//     }

//     pub fn clear_row(&mut self, row: usize) {
//         self.update_row(row, [Cell::Empty; GRID_WIDTH]);
//     }

//     pub fn clear_and_shift_row(&mut self, row: usize) {
//         self.clear_row(row);
//         for index in (1..=row).rev() {
//             self.shift_row_down(index);
//         }
//     }

//     pub fn is_game_over(&self) -> bool {
//         // If there is an occupied cell on the top row of the grid, the game is over
//         self.grid[0]
//             .iter()
//             .any(|&cell| matches!(cell, Cell::Occupied))
//     }
// }

// impl RenderOnce for Tetris {
//     fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
//         div()
//             .size_full()
//             .items_center()
//             .flex()
//             .flex_col()
//             .gap_px()
//             .children(
//                 self.grid
//                     .iter()
//                     .map(|row| {
//                         div().flex().gap_px().children(
//                             row.iter()
//                                 .map(|cell| match cell {
//                                     Cell::Empty => div().w_8().h_8().bg(gpui::red()),
//                                     Cell::Occupied => div().w_8().h_8().bg(gpui::blue()),
//                                 })
//                                 .collect::<Vec<_>>(),
//                         )
//                     })
//                     .collect::<Vec<_>>(),
//             )
//     }
// }
