use gpui::SharedString;
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

#[derive(IntoElement, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockStatus {
    Empty,
    Occupied,
}

impl RenderOnce for BlockStatus {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        div()
            .size_6()
            .border()
            .border_color(gpui::white())
            .when(self == Self::Occupied, |this| this.bg(gpui::white()))
    }
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

#[derive(IntoElement, Serialize, Deserialize)]
pub struct Grid {
    grid_blocks: [[BlockStatus; GRID_WIDTH]; GRID_HEIGHT],
}

impl RenderOnce for Grid {
    fn render(self, _cx: &mut WindowContext) -> impl IntoElement {
        div()
            .border()
            .border_color(gpui::white())
            .p_2()
            .flex()
            .flex_col()
            .gap_1()
            .children(self.grid_blocks.iter().map(|row| {
                div().flex_none().flex().gap_1().children(
                    row.iter()
                        .map(|block| div().flex_none().gap_1().child(block.to_owned())),
                )
            }))
    }
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
}

#[derive(IntoElement)]
pub struct Game {
    grid: Grid,
    score: usize,
}

impl RenderOnce for Game {
    fn render(self, cx: &mut WindowContext) -> impl IntoElement {
        let score: SharedString = format!("Score: {}", self.score).to_string().into();

        div()
            .w_96()
            .h_96()
            .bg(gpui::black())
            .p_1()
            .flex()
            .flex_col()
            .gap_1()
            .child("GPUI Tetris")
            .child(score)
            .child(self.grid);
    }
}

impl Game {
    pub fn new() -> Self {
        Self {
            grid: Grid::new(),
            score: 0,
        }
    }

    fn process_input(&mut self, action: PlayerAction) {
        match action {
            PlayerAction::MoveLeft => self.move_tetromino(-1, 0),
            PlayerAction::MoveRight => self.move_tetromino(1, 0),
            PlayerAction::MoveDown => self.move_tetromino(0, 1),
            PlayerAction::Drop => self.drop_tetromino(),
            PlayerAction::Rotate => self.rotate_tetromino(),
            PlayerAction::Pause => self.toggle_pause(),
            PlayerAction::Restart => self.restart_game(),
        }
    }

    pub fn update_game_state(&mut self) {
        // This is a simplified example of where you would have your game loop.
        // In a real implementation, this would be repeatedly called as
        // part of your main game loop, and input would be handled each time.

        // Get the player's input (in a real game loop, input would be fetched dynamically)
        let action = get_player_input();

        // Process the input
        self.process_input(action);

        // Other game update logic here...
    }

    fn move_tetromino(&mut self, dx: isize, dy: isize) {
        // Attempt to move the current tetromino by `dx` and `dy`
        unimplemented!("Move tetromino not implemented")
    }

    fn drop_tetromino(&mut self) {
        // Drop the current tetromino to the bottom of the grid
        unimplemented!("Drop tetromino not implemented")
    }

    fn rotate_tetromino(&mut self) {
        // Rotate the current tetromino
        unimplemented!("Rotate tetromino not implemented")
    }

    fn toggle_pause(&mut self) {
        // Pause or resume the game
        unimplemented!("Pause/resume game not implemented")
    }

    fn restart_game(&mut self) {
        // Restart the game, reset the game state
        unimplemented!("Restart game not implemented")
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum PlayerAction {
    MoveLeft,
    MoveRight,
    MoveDown,
    Drop,
    Rotate,
    Pause,
    Restart,
}

fn get_player_input() -> PlayerAction {
    // This is where the actual key press handling logic would be,
    // but for now, we'll return a hardcoded action.
    // In an actual game loop, we'd call a library-specific function here,
    // like listening for a key event in a while loop.

    // Example hardcoded player action:
    PlayerAction::MoveDown
}
