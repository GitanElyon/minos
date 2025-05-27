use pyo3::prelude::*;
use std::collections::HashMap;

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;

#[derive(Clone, Debug)]
#[pyclass]
pub struct TetrisBoard {
    #[pyo3(get, set)]
    pub grid: Vec<Vec<i32>>, // 0 = empty, 1-7 = piece types
    #[pyo3(get, set)]
    pub current_piece: Option<String>,
    #[pyo3(get, set)]
    pub held_piece: Option<String>,
    #[pyo3(get, set)]
    pub next_pieces: Vec<Option<String>>,
    #[pyo3(get, set)]
    pub can_hold: bool, // Can only hold once per piece
}

#[pymethods]
impl TetrisBoard {
    #[new]
    fn new() -> Self {
        TetrisBoard {
            grid: vec![vec![0; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: None,
            held_piece: None,
            next_pieces: vec![],
            can_hold: true,
        }
    }
    
    fn update_pieces(&mut self, current: Option<String>, held: Option<String>, next: Vec<Option<String>>) {
        self.current_piece = current;
        self.held_piece = held;
        self.next_pieces = next;
    }
    
    fn is_valid_position(&self, piece: &str, x: i32, y: i32, rotation: u8) -> bool {
        let shape = get_piece_shape(piece, rotation);
        
        for (dy, row) in shape.iter().enumerate() {
            for (dx, &cell) in row.iter().enumerate() {
                if cell != 0 {
                    let board_x = x + dx as i32;
                    let board_y = y + dy as i32;
                    
                    // Check bounds
                    if board_x < 0 || board_x >= BOARD_WIDTH as i32 || 
                       board_y < 0 || board_y >= BOARD_HEIGHT as i32 {
                        return false;
                    }
                    
                    // Check collision with existing pieces
                    if self.grid[board_y as usize][board_x as usize] != 0 {
                        return false;
                    }
                }
            }
        }
        true
    }
    
    fn place_piece(&mut self, piece: &str, x: i32, y: i32, rotation: u8) -> bool {
        if !self.is_valid_position(piece, x, y, rotation) {
            return false;
        }
        
        let shape = get_piece_shape(piece, rotation);
        let piece_id = get_piece_id(piece);
        
        for (dy, row) in shape.iter().enumerate() {
            for (dx, &cell) in row.iter().enumerate() {
                if cell != 0 {
                    let board_x = (x + dx as i32) as usize;
                    let board_y = (y + dy as i32) as usize;
                    self.grid[board_y][board_x] = piece_id;
                }
            }
        }
        
        // Check for line clears
        self.clear_lines();
        true
    }
    
    fn clear_lines(&mut self) -> u32 {
        let mut lines_cleared = 0;
        let mut y = BOARD_HEIGHT as i32 - 1;
        
        while y >= 0 {
            if self.is_line_full(y as usize) {
                self.grid.remove(y as usize);
                self.grid.insert(0, vec![0; BOARD_WIDTH]);
                lines_cleared += 1;
            } else {
                y -= 1;
            }
        }
        
        lines_cleared
    }
    
    fn is_line_full(&self, y: usize) -> bool {
        self.grid[y].iter().all(|&cell| cell != 0)
    }
    
    fn get_drop_position(&self, piece: &str, x: i32, rotation: u8) -> i32 {
        let mut drop_y = 0;
        
        for y in 0..BOARD_HEIGHT as i32 {
            if self.is_valid_position(piece, x, y, rotation) {
                drop_y = y;
            } else {
                break;
            }
        }
        
        drop_y
    }

    // Add this method to your existing TetrisBoard implementation
    fn print_board(&self) {
        println!("Current board state:");
        for y in 0..BOARD_HEIGHT {
            print!("  ");
            for x in 0..BOARD_WIDTH {
                let cell = self.grid[y][x];
                if cell == 0 {
                    print!(".");
                } else {
                    print!("{}", cell);
                }
            }
            println!();
        }
        println!();
    }
    
    fn print_board_with_piece(&self, piece: &str, x: i32, y: i32, rotation: u8) {
        let mut display_grid = self.grid.clone();
        let shape = get_piece_shape(piece, rotation);
        let piece_id = get_piece_id(piece);
        
        // Add piece to display grid
        for (dy, row) in shape.iter().enumerate() {
            for (dx, &cell) in row.iter().enumerate() {
                if cell != 0 {
                    let board_x = x + dx as i32;
                    let board_y = y + dy as i32;
                    
                    if board_x >= 0 && board_x < BOARD_WIDTH as i32 && 
                       board_y >= 0 && board_y < BOARD_HEIGHT as i32 {
                        display_grid[board_y as usize][board_x as usize] = piece_id + 10; // Use 10+ to distinguish
                    }
                }
            }
        }
        
        println!("Board with piece {} at x={}, y={}, rot={}:", piece, x, y, rotation);
        for y in 0..BOARD_HEIGHT {
            print!("  ");
            for x in 0..BOARD_WIDTH {
                let cell = display_grid[y][x];
                if cell == 0 {
                    print!(".");
                } else if cell > 10 {
                    print!("*"); // Show the falling piece
                } else {
                    print!("{}", cell);
                }
            }
            println!();
        }
        println!();
    }
}

// Piece definitions
fn get_piece_shape(piece: &str, rotation: u8) -> Vec<Vec<i32>> {
    match piece {
        "I" => match rotation % 2 {
            0 => vec![vec![1, 1, 1, 1]],
            _ => vec![vec![1], vec![1], vec![1], vec![1]],
        },
        "O" => vec![vec![1, 1], vec![1, 1]], // O piece doesn't rotate
        "T" => match rotation % 4 {
            0 => vec![vec![0, 1, 0], vec![1, 1, 1]],
            1 => vec![vec![1, 0], vec![1, 1], vec![1, 0]],
            2 => vec![vec![1, 1, 1], vec![0, 1, 0]],
            _ => vec![vec![0, 1], vec![1, 1], vec![0, 1]],
        },
        "S" => match rotation % 2 {
            0 => vec![vec![0, 1, 1], vec![1, 1, 0]],
            _ => vec![vec![1, 0], vec![1, 1], vec![0, 1]],
        },
        "Z" => match rotation % 2 {
            0 => vec![vec![1, 1, 0], vec![0, 1, 1]],
            _ => vec![vec![0, 1], vec![1, 1], vec![1, 0]],
        },
        "J" => match rotation % 4 {
            0 => vec![vec![1, 0, 0], vec![1, 1, 1]],
            1 => vec![vec![1, 1], vec![1, 0], vec![1, 0]],
            2 => vec![vec![1, 1, 1], vec![0, 0, 1]],
            _ => vec![vec![0, 1], vec![0, 1], vec![1, 1]],
        },
        "L" => match rotation % 4 {
            0 => vec![vec![0, 0, 1], vec![1, 1, 1]],
            1 => vec![vec![1, 0], vec![1, 0], vec![1, 1]],
            2 => vec![vec![1, 1, 1], vec![1, 0, 0]],
            _ => vec![vec![1, 1], vec![0, 1], vec![0, 1]],
        },
        _ => vec![vec![1]], // Default single block
    }
}

fn get_piece_id(piece: &str) -> i32 {
    match piece {
        "I" => 1,
        "O" => 2,
        "T" => 3,
        "S" => 4,
        "Z" => 5,
        "J" => 6,
        "L" => 7,
        _ => 1,
    }
}

#[derive(Clone, Debug)]
#[pyclass]
pub struct Move {
    #[pyo3(get, set)]
    pub piece: String,
    #[pyo3(get, set)]
    pub x: i32,
    #[pyo3(get, set)]
    pub y: i32,
    #[pyo3(get, set)]
    pub rotation: u8,
    #[pyo3(get, set)]
    pub score: f64,
}

#[pymethods]
impl Move {
    #[new]
    fn new(piece: String, x: i32, y: i32, rotation: u8, score: f64) -> Self {
        Move { piece, x, y, rotation, score }
    }
}

/// Calculate the best move for the current piece
#[pyfunction]
fn calculate_best_move(
    board: &mut TetrisBoard,
    piece: Option<String>,
) -> PyResult<Option<Move>> {
    if let Some(piece_type) = piece {
        let mut best_move = None;
        let mut best_score = f64::NEG_INFINITY;
        
        // Try all possible positions and rotations
        for rotation in 0..4 {
            for x in 0..BOARD_WIDTH as i32 {
                if board.is_valid_position(&piece_type, x, 0, rotation) {
                    let y = board.get_drop_position(&piece_type, x, rotation);
                    
                    // Create a copy of the board to test the move
                    let mut test_board = board.clone();
                    if test_board.place_piece(&piece_type, x, y, rotation) {
                        let score = evaluate_board(&test_board);
                        
                        if score > best_score {
                            best_score = score;
                            best_move = Some(Move::new(piece_type.clone(), x, y, rotation, score));
                        }
                    }
                }
            }
        }
        
        Ok(best_move)
    } else {
        Ok(None)
    }
}

/// Calculate the best move for the current piece with full debugging
#[pyfunction]
fn calculate_best_move_debug(
    board: &mut TetrisBoard,
    piece: Option<String>,
) -> PyResult<Option<Move>> {
    println!("=== RUST AI DEBUG ===");
    
    if let Some(piece_type) = piece {
        println!("Calculating best move for piece: {}", piece_type);
        
        // Print current board state
        board.print_board();
        
        let mut best_move = None;
        let mut best_score = f64::NEG_INFINITY;
        let mut move_count = 0;
        
        println!("Trying all possible moves:");
        
        // Try all possible positions and rotations
        for rotation in 0..4 {
            for x in 0..BOARD_WIDTH as i32 {
                if board.is_valid_position(&piece_type, x, 0, rotation) {
                    let y = board.get_drop_position(&piece_type, x, rotation);
                    
                    // Create a copy of the board to test the move
                    let mut test_board = board.clone();
                    if test_board.place_piece(&piece_type, x, y, rotation) {
                        let score = evaluate_board_debug(&test_board, &piece_type, x, y, rotation);
                        
                        move_count += 1;
                        println!("  Move {}: x={}, rot={}, score={:.2}", move_count, x, rotation, score);
                        
                        if score > best_score {
                            best_score = score;
                            best_move = Some(Move::new(piece_type.clone(), x, y, rotation, score));
                            println!("    ^ NEW BEST MOVE!");
                        }
                    }
                }
            }
        }
        
        if let Some(ref mv) = best_move {
            println!("\nFinal decision: x={}, rot={}, score={:.2}", mv.x, mv.rotation, mv.score);
            
            // Show what the board would look like
            let mut preview_board = board.clone();
            preview_board.place_piece(&piece_type, mv.x, mv.y, mv.rotation);
            println!("Board after move:");
            preview_board.print_board();
        } else {
            println!("No valid moves found!");
        }
        
        println!("=== END DEBUG ===\n");
        
        Ok(best_move)
    } else {
        println!("No piece provided to calculate_best_move_debug");
        Ok(None)
    }
}

/// Evaluate board position (higher is better)
fn evaluate_board(board: &TetrisBoard) -> f64 {
    let mut score = 0.0;
    
    // Weights found through genetic algorithms/tuning
    let line_clear_weight = 760.666;
    let hole_weight = -35.0;
    let bumpiness_weight = -18.0;
    let height_weight = -51.0;
    
    // Calculate metrics
    let lines_cleared = count_complete_lines(board) as f64;
    let holes = count_holes(board) as f64;
    let bumpiness = get_bumpiness(board) as f64;
    let aggregate_height = get_aggregate_height(board) as f64;
    
    // Calculate final score
    score += lines_cleared * line_clear_weight;
    score += holes * hole_weight;
    score += bumpiness * bumpiness_weight;
    score += aggregate_height * height_weight;
    
    score
}

/// Evaluate board position with debugging output
fn evaluate_board_debug(board: &TetrisBoard, piece: &str, x: i32, y: i32, rotation: u8) -> f64 {
    // Weights
    let line_clear_weight = 760.666;
    let hole_weight = -35.0;
    let bumpiness_weight = -18.0;
    let height_weight = -51.0;
    
    // Calculate metrics
    let lines_cleared = count_complete_lines(board) as f64;
    let holes = count_holes(board) as f64;
    let bumpiness = get_bumpiness(board) as f64;
    let aggregate_height = get_aggregate_height(board) as f64;
    
    // Calculate components
    let line_score = lines_cleared * line_clear_weight;
    let hole_score = holes * hole_weight;
    let bump_score = bumpiness * bumpiness_weight;
    let height_score = aggregate_height * height_weight;
    
    let total_score = line_score + hole_score + bump_score + height_score;
    
    // Only print details for moves that seem promising or if there are few moves
    if total_score > -1000.0 || lines_cleared > 0.0 {
        println!("    Details - Lines:{:.0} ({:.1}) Holes:{:.0} ({:.1}) Bump:{:.0} ({:.1}) Height:{:.0} ({:.1})",
                lines_cleared, line_score,
                holes, hole_score, 
                bumpiness, bump_score,
                aggregate_height, height_score);
    }
    
    total_score
}

fn get_column_heights(board: &TetrisBoard) -> Vec<u32> {
    let mut heights = vec![0; BOARD_WIDTH];
    
    for x in 0..BOARD_WIDTH {
        for y in 0..BOARD_HEIGHT {
            if board.grid[y][x] != 0 {
                heights[x] = (BOARD_HEIGHT - y) as u32;
                break;
            }
        }
    }
    
    heights
}

fn get_aggregate_height(board: &TetrisBoard) -> u32 {
    get_column_heights(board).iter().sum()
}

fn count_holes(board: &TetrisBoard) -> u32 {
    let mut holes = 0;
    
    for x in 0..BOARD_WIDTH {
        let mut found_block = false;
        for y in 0..BOARD_HEIGHT {
            if board.grid[y][x] != 0 {
                found_block = true;
            } else if found_block {
                holes += 1;
            }
        }
    }
    
    holes
}

fn get_bumpiness(board: &TetrisBoard) -> u32 {
    let heights = get_column_heights(board);
    let mut bumpiness = 0;
    
    for i in 0..BOARD_WIDTH - 1 {
        bumpiness += (heights[i] as i32 - heights[i + 1] as i32).abs() as u32;
    }
    
    bumpiness
}

fn count_complete_lines(board: &TetrisBoard) -> u32 {
    let mut complete_lines = 0;
    
    for y in 0..BOARD_HEIGHT {
        if board.is_line_full(y) {
            complete_lines += 1;
        }
    }
    
    complete_lines
}

/// Initialize a new Tetris board
#[pyfunction]
fn create_board() -> PyResult<TetrisBoard> {
    Ok(TetrisBoard::new())
}

// Add to your lib.rs

// Global board state - we'll keep one instance
static mut GAME_BOARD: Option<TetrisBoard> = None;

/// Initialize the game board (call once at start)
#[pyfunction]
fn initialize_game_board() -> PyResult<()> {
    unsafe {
        GAME_BOARD = Some(TetrisBoard::new());
    }
    println!("Game board initialized");
    Ok(())
}

/// Get the current board state for debugging
#[pyfunction] 
fn get_board_state() -> PyResult<TetrisBoard> {
    unsafe {
        match &GAME_BOARD {
            Some(board) => Ok(board.clone()),
            None => {
                println!("Board not initialized, creating new one");
                let board = TetrisBoard::new();
                GAME_BOARD = Some(board.clone());
                Ok(board)
            }
        }
    }
}

/// Update pieces without resetting board
#[pyfunction]
fn update_game_pieces(
    current: Option<String>, 
    held: Option<String>, 
    next: Vec<Option<String>>
) -> PyResult<()> {
    unsafe {
        if let Some(board) = &mut GAME_BOARD {
            board.update_pieces(current, held, next);
        }
    }
    Ok(())
}

/// Calculate best move using the persistent board state
#[pyfunction]
fn calculate_best_move_persistent(
    piece: Option<String>,
) -> PyResult<Option<Move>> {
    unsafe {
        if let Some(board) = &mut GAME_BOARD {
            return calculate_best_move_debug(board, piece);
        }
    }
    
    // Fallback - create new board if none exists
    println!("No persistent board found, creating temporary one");
    let mut board = TetrisBoard::new();
    calculate_best_move_debug(&mut board, piece)
}

/// Execute a move on the persistent board
#[pyfunction]
fn execute_move_on_board(mv: &Move) -> PyResult<bool> {
    unsafe {
        if let Some(board) = &mut GAME_BOARD {
            println!("Executing move on persistent board: {} at x={}, rot={}", 
                    mv.piece, mv.x, mv.rotation);
                    
            // Show board before
            println!("Board before move:");
            board.print_board();
            
            let success = board.place_piece(&mv.piece, mv.x, mv.y, mv.rotation);
            
            if success {
                println!("Move executed successfully!");
                println!("Board after move:");
                board.print_board();
            } else {
                println!("Move execution failed!");
            }
            
            return Ok(success);
        }
    }
    
    println!("No persistent board available");
    Ok(false)
}

/// Reset the board (for new games)
#[pyfunction]
fn reset_game_board() -> PyResult<()> {
    unsafe {
        GAME_BOARD = Some(TetrisBoard::new());
    }
    println!("Game board reset");
    Ok(())
}

// Update these functions in your lib.rs

/// Get the standard spawn position for each piece type (CORRECTED)
fn get_spawn_position(piece: &str) -> (i32, i32) {
    // Standard Tetris spawn positions (center of board)
    match piece {
        "I" => (3, 0),  // I-piece: 4 blocks wide, spawn at 3 so it occupies 3,4,5,6
        "O" => (4, 0),  // O-piece: 2x2, spawn at 4 so it occupies 4,5
        "T" => (3, 0),  // T-piece: 3 wide, spawn at 3 so center is at 4
        "S" => (3, 0),  // S-piece: 3 wide, spawn at 3 so center is at 4
        "Z" => (3, 0),  // Z-piece: 3 wide, spawn at 3 so center is at 4
        "J" => (3, 0),  // J-piece: 3 wide, spawn at 3 so center is at 4
        "L" => (3, 0),  // L-piece: 3 wide, spawn at 3 so center is at 4
        _ => (3, 0),    // Default
    }
}

/// Convert Rust move to a series of input commands
#[derive(Clone, Debug)]
#[pyclass]
pub struct InputCommand {
    #[pyo3(get, set)]
    pub action: String,  // "left", "right", "rotate_cw", "rotate_ccw", "drop", "hold"
    #[pyo3(get, set)]
    pub count: i32,      // How many times to perform this action
}

#[pymethods]
impl InputCommand {
    #[new]
    fn new(action: String, count: i32) -> Self {
        InputCommand { action, count }
    }
}

/// Calculate the input commands needed to execute a move (with 180-degree optimization)
#[pyfunction]
fn calculate_input_commands(
    piece: &str,
    target_move: &Move
) -> PyResult<Vec<InputCommand>> {
    let mut commands = Vec::new();
    
    // Get spawn position
    let (spawn_x, _spawn_y) = get_spawn_position(piece);
    
    println!("Calculating inputs for {} from spawn x={} to target x={}, rotation={}", 
             piece, spawn_x, target_move.x, target_move.rotation);
    
    // Step 1: Handle rotations with 180-degree optimization
    let rotation = target_move.rotation % 4;
    
    if rotation == 2 {
        // Use 180-degree flip for efficiency
        commands.push(InputCommand::new("rotate_180".to_string(), 1));
        println!("  Command: Rotate 180 degrees (1 flip)");
    } else if rotation > 0 {
        commands.push(InputCommand::new("rotate_cw".to_string(), rotation as i32));
        println!("  Command: Rotate clockwise {} times", rotation);
    }
    
    // Step 2: Handle horizontal movement
    let moves_needed = target_move.x - spawn_x;
    
    if moves_needed > 0 {
        commands.push(InputCommand::new("right".to_string(), moves_needed));
        println!("  Command: Move right {} times", moves_needed);
    } else if moves_needed < 0 {
        commands.push(InputCommand::new("left".to_string(), -moves_needed));
        println!("  Command: Move left {} times", -moves_needed);
    }
    
    // Step 3: Drop the piece
    commands.push(InputCommand::new("drop".to_string(), 1));
    println!("  Command: Drop piece");
    
    Ok(commands)
}


/// Get the best move and the input commands to execute it
#[pyfunction]
fn get_optimal_move_with_inputs(
    piece: Option<String>,
) -> PyResult<Option<(Move, Vec<InputCommand>)>> {
    if let Some(piece_type) = piece {
        // Calculate the best move
        if let Some(best_move) = calculate_best_move_persistent(Some(piece_type.clone()))? {
            // Calculate the input commands needed
            let commands = calculate_input_commands(&piece_type, &best_move)?;
            
            println!("Optimal move calculated:");
            println!("  Move: {} at x={}, rotation={}, score={:.1}", 
                     best_move.piece, best_move.x, best_move.rotation, best_move.score);
            println!("  Commands: {} steps", commands.len());
            
            return Ok(Some((best_move, commands)));
        }
    }
    
    Ok(None)
}

/// Test if a move from spawn position is valid
#[pyfunction]
fn test_spawn_position(piece: String) -> PyResult<bool> {
    let (spawn_x, spawn_y) = get_spawn_position(&piece);
    
    unsafe {
        if let Some(board) = &GAME_BOARD {
            let is_valid = board.is_valid_position(&piece, spawn_x, spawn_y, 0);
            println!("Testing spawn for {}: x={}, y={}, valid={}", piece, spawn_x, spawn_y, is_valid);
            return Ok(is_valid);
        }
    }
    
    Ok(false)
}

// Add to your lib.rs

/// Get the width of a piece at a specific rotation
fn get_piece_width(piece: &str, rotation: u8) -> i32 {
    let shape = get_piece_shape(piece, rotation);
    shape[0].len() as i32
}

/// Get the height of a piece at a specific rotation
fn get_piece_height(piece: &str, rotation: u8) -> i32 {
    get_piece_shape(piece, rotation).len() as i32
}

/// Get the adjusted spawn position accounting for piece width after rotation
fn get_adjusted_spawn_position(piece: &str, rotation: u8) -> (i32, i32) {
    let base_spawn = get_spawn_position(piece);
    let piece_width = get_piece_width(piece, rotation);
    
    // Adjust x position to keep piece centered
    let adjusted_x = match piece {
        "I" => {
            match rotation % 2 {
                0 => 3,  // Horizontal I-piece (4 wide)
                _ => 5,  // Vertical I-piece (1 wide)
            }
        },
        "O" => 4,  // O-piece is always 2x2, same position
        _ => {
            // For other pieces, center them based on their rotated width
            let center_x = 5; // Board center
            center_x - (piece_width / 2)
        }
    };
    
    (adjusted_x, base_spawn.1)
}

/// Calculate positions accounting for rotation-dependent piece dimensions (FIXED)
#[pyfunction]
fn calculate_input_commands_fixed(
    piece: &str,
    target_move: &Move
) -> PyResult<Vec<InputCommand>> {
    let mut commands = Vec::new();
    
    // Always use the standard spawn position (don't change this)
    let (spawn_x, _spawn_y) = get_spawn_position(piece);
    
    println!("Calculating inputs for {} from spawn x={} to target x={}, rotation={}", 
             piece, spawn_x, target_move.x, target_move.rotation);
    
    // Step 1: Handle rotations first
    let rotation = target_move.rotation % 4;
    
    if rotation == 2 {
        commands.push(InputCommand::new("rotate_180".to_string(), 1));
        println!("  Command: Rotate 180 degrees");
    } else if rotation > 0 {
        commands.push(InputCommand::new("rotate_cw".to_string(), rotation as i32));
        println!("  Command: Rotate clockwise {} times", rotation);
    }
    
    // Step 2: Calculate movement needed, accounting for rotation offset
    let mut effective_spawn_x = spawn_x;
    
    // Adjust effective spawn position based on how rotation changes the piece position
    if rotation > 0 {
        effective_spawn_x = match piece {
            "I" => {
                match rotation % 2 {
                    0 => spawn_x,      // Horizontal I-piece, no adjustment
                    _ => spawn_x + 1,  // Vertical I-piece shifts right by 1
                }
            },
            "O" => spawn_x,  // O-piece doesn't change position when rotated
            "T" | "S" | "Z" | "J" | "L" => {
                // These pieces might shift slightly when rotated
                match rotation {
                    1 => spawn_x,      // First rotation
                    2 => spawn_x,      // 180 degrees
                    3 => spawn_x,      // Third rotation
                    _ => spawn_x,
                }
            },
            _ => spawn_x,
        };
    }
    
    let moves_needed = target_move.x - effective_spawn_x;
    
    println!("  Effective spawn after rotation: {}, moves needed: {}", effective_spawn_x, moves_needed);
    
    if moves_needed > 0 {
        commands.push(InputCommand::new("right".to_string(), moves_needed));
        println!("  Command: Move right {} times", moves_needed);
    } else if moves_needed < 0 {
        commands.push(InputCommand::new("left".to_string(), -moves_needed));
        println!("  Command: Move left {} times", -moves_needed);
    }
    
    // Step 3: Drop the piece
    commands.push(InputCommand::new("drop".to_string(), 1));
    println!("  Command: Drop piece");
    
    Ok(commands)
}

/// Enhanced move calculation that considers piece dimensions at each rotation
#[pyfunction]
fn get_optimal_move_with_inputs_fixed(
    piece: Option<String>,
) -> PyResult<Option<(Move, Vec<InputCommand>)>> {
    if let Some(piece_type) = piece {
        // Calculate the best move
        if let Some(best_move) = calculate_best_move_persistent(Some(piece_type.clone()))? {
            // Calculate the input commands with rotation-aware positioning
            let commands = calculate_input_commands_fixed(&piece_type, &best_move)?;
            
            println!("Optimal move calculated:");
            println!("  Move: {} at x={}, rotation={}, score={:.1}", 
                     best_move.piece, best_move.x, best_move.rotation, best_move.score);
            println!("  Commands: {} steps", commands.len());
            
            return Ok(Some((best_move, commands)));
        }
    }
    
    Ok(None)
}

/// Simple debug to track position shifts during rotation
#[pyfunction]
fn debug_rotation_position_shift(piece: String, rotation: u8) -> PyResult<()> {
    println!("=== ROTATION POSITION DEBUG ===");
    println!("Piece: {}, Rotation: {}", piece, rotation);
    
    let (spawn_x, spawn_y) = get_spawn_position(&piece);
    println!("Spawn position: x={}, y={}", spawn_x, spawn_y);
    
    // Test if piece can be placed at spawn with this rotation
    unsafe {
        if let Some(board) = &GAME_BOARD {
            let can_place = board.is_valid_position(&piece, spawn_x, spawn_y, rotation);
            println!("Can place at spawn with rotation {}: {}", rotation, can_place);
            
            // Try different x positions to see where it can actually be placed
            for test_x in (spawn_x - 3)..(spawn_x + 4) {
                if board.is_valid_position(&piece, test_x, spawn_y, rotation) {
                    println!("  Can place at x={} with rotation {}", test_x, rotation);
                }
            }
        }
    }
    println!("=== END DEBUG ===");
    Ok(())
}

/// Calculate input commands with position correction for rotation
#[pyfunction]
fn calculate_input_commands_corrected(
    piece: &str,
    target_move: &Move
) -> PyResult<Vec<InputCommand>> {
    let mut commands = Vec::new();
    
    let (spawn_x, _spawn_y) = get_spawn_position(piece);
    
    println!("Calculating inputs for {} from spawn x={} to target x={}, rotation={}", 
             piece, spawn_x, target_move.x, target_move.rotation);
    
    // Step 1: Handle rotations first
    let rotation = target_move.rotation % 4;
    
    if rotation == 2 {
        commands.push(InputCommand::new("rotate_180".to_string(), 1));
        println!("  Command: Rotate 180 degrees");
    } else if rotation > 0 {
        commands.push(InputCommand::new("rotate_cw".to_string(), rotation as i32));
        println!("  Command: Rotate clockwise {} times", rotation);
    }
    
    // Step 2: Calculate the actual position the piece ends up at after rotation
    let actual_x_after_rotation = get_actual_position_after_rotation(piece, spawn_x, rotation);
    
    // Step 3: Calculate movement needed from the actual position to target
    let moves_needed = target_move.x - actual_x_after_rotation;
    
    println!("  After rotation, piece is at x={}, need to move {} to reach target x={}", 
             actual_x_after_rotation, moves_needed, target_move.x);
    
    if moves_needed > 0 {
        commands.push(InputCommand::new("right".to_string(), moves_needed));
        println!("  Command: Move right {} times", moves_needed);
    } else if moves_needed < 0 {
        commands.push(InputCommand::new("left".to_string(), -moves_needed));
        println!("  Command: Move left {} times", -moves_needed);
    }
    
    commands.push(InputCommand::new("drop".to_string(), 1));
    println!("  Command: Drop piece");
    
    Ok(commands)
}

/// Calculate where a piece actually ends up after rotation
fn get_actual_position_after_rotation(piece: &str, spawn_x: i32, rotation: u8) -> i32 {
    // This function needs to account for how your specific game handles rotation
    // These values might need to be adjusted based on testing
    
    match piece {
        "I" => {
            match rotation % 2 {
                0 => spawn_x,      // Horizontal I (4 wide)
                _ => spawn_x + 1,  // Vertical I (1 wide) - often shifts right
            }
        },
        "O" => spawn_x,  // O piece doesn't change position when rotated
        "T" => {
            match rotation % 4 {
                0 => spawn_x,      // Original orientation
                1 => spawn_x,      // Right turn - might shift
                2 => spawn_x,      // Upside down
                3 => spawn_x,      // Left turn - might shift
                _ => spawn_x,
            }
        },
        "S" | "Z" => {
            match rotation % 2 {
                0 => spawn_x,      // Horizontal
                _ => spawn_x,      // Vertical - might shift
            }
        },
        "J" | "L" => {
            match rotation % 4 {
                0 => spawn_x,      // Original
                1 => spawn_x,      // Rotated
                2 => spawn_x,      // Flipped
                3 => spawn_x,      // Rotated other way
                _ => spawn_x,
            }
        },
        _ => spawn_x,
    }
}

/// Python module definition
#[pymodule]
fn tetris_bot_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(calculate_best_move, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_best_move_debug, m)?)?;  // Add this line
    m.add_function(wrap_pyfunction!(create_board, m)?)?;
    
    // Add new persistent board functions
    m.add_function(wrap_pyfunction!(initialize_game_board, m)?)?;
    m.add_function(wrap_pyfunction!(get_board_state, m)?)?;
    m.add_function(wrap_pyfunction!(update_game_pieces, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_best_move_persistent, m)?)?;
    m.add_function(wrap_pyfunction!(execute_move_on_board, m)?)?;
    m.add_function(wrap_pyfunction!(reset_game_board, m)?)?;
    
    // New input calculation functions
    m.add_function(wrap_pyfunction!(calculate_input_commands, m)?)?;
    m.add_function(wrap_pyfunction!(get_optimal_move_with_inputs, m)?)?;
    m.add_function(wrap_pyfunction!(test_spawn_position, m)?)?;
    
    // Add the fixed functions
    m.add_function(wrap_pyfunction!(calculate_input_commands_fixed, m)?)?;
    m.add_function(wrap_pyfunction!(get_optimal_move_with_inputs_fixed, m)?)?;
    
    // Add the debug function
    m.add_function(wrap_pyfunction!(debug_rotation_position_shift, m)?)?;
    
    // Add the corrected input commands function
    m.add_function(wrap_pyfunction!(calculate_input_commands_corrected, m)?)?;
    
    m.add_class::<TetrisBoard>()?;
    m.add_class::<Move>()?;
    m.add_class::<InputCommand>()?;
    Ok(())
}