use pyo3::prelude::*;

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;

#[derive(Clone, Debug)]
#[pyclass]
pub struct TetrisBoard {
    #[pyo3(get, set)]
    pub grid: Vec<Vec<i32>>,
    #[pyo3(get, set)]
    pub current_piece: Option<String>,
    #[pyo3(get, set)]
    pub held_piece: Option<String>,
    #[pyo3(get, set)]
    pub next_pieces: Vec<Option<String>>,
    #[pyo3(get, set)]
    pub can_hold: bool,
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
                    
                    if board_x < 0 || board_x >= BOARD_WIDTH as i32 || 
                       board_y < 0 || board_y >= BOARD_HEIGHT as i32 {
                        return false;
                    }
                    
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

#[derive(Clone, Debug)]
#[pyclass]
pub struct InputCommand {
    #[pyo3(get, set)]
    pub action: String,
    #[pyo3(get, set)]
    pub count: i32,
}

#[pymethods]
impl InputCommand {
    #[new]
    fn new(action: String, count: i32) -> Self {
        InputCommand { action, count }
    }
}

// Piece definitions
fn get_piece_shape(piece: &str, rotation: u8) -> Vec<Vec<i32>> {
    match piece {
        "I" => match rotation % 2 {
            0 => vec![vec![1, 1, 1, 1]],
            _ => vec![vec![1], vec![1], vec![1], vec![1]],
        },
        "O" => vec![vec![1, 1], vec![1, 1]],
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
        _ => vec![vec![1]],
    }
}

fn get_piece_id(piece: &str) -> i32 {
    match piece {
        "I" => 1, "O" => 2, "T" => 3, "S" => 4, "Z" => 5, "J" => 6, "L" => 7,
        _ => 1,
    }
}

fn get_spawn_position(piece: &str) -> (i32, i32) {
    match piece {
        "I" => (3, 0),
        "O" => (4, 0),
        _ => (3, 0),
    }
}

// Board evaluation
fn evaluate_board(board: &TetrisBoard) -> f64 {
    let lines_cleared = count_complete_lines(board) as f64;
    let holes = count_holes(board) as f64;
    let bumpiness = get_bumpiness(board) as f64;
    let height = get_aggregate_height(board) as f64;
    
    lines_cleared * 750.0 - holes * 350.0 - bumpiness * 180.0 - height * 510.0
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

// Input calculation
// Update the calculate_input_commands function with optimized rotations
fn calculate_input_commands(piece: &str, target_move: &Move) -> PyResult<Vec<InputCommand>> {
    let mut commands = Vec::new();
    let (spawn_x, _) = get_spawn_position(piece);
    
    // Handle rotations with optimization
    let rotation = target_move.rotation % 4;
    
    match rotation {
        0 => {
            // No rotation needed
        },
        1 => {
            // 1 rotation clockwise
            commands.push(InputCommand::new("rotate_cw".to_string(), 1));
        },
        2 => {
            // 2 rotations - use 180 flip if available, otherwise 2 clockwise
            commands.push(InputCommand::new("rotate_180".to_string(), 1));
        },
        3 => {
            // 3 rotations clockwise = 1 rotation counter-clockwise
            commands.push(InputCommand::new("rotate_ccw".to_string(), 1));
        },
        _ => {}
    }
    
    // Calculate movement after rotation
    let actual_position = get_actual_position_after_rotation(piece, spawn_x, rotation);
    let moves_needed = target_move.x - actual_position;
    
    if moves_needed > 0 {
        commands.push(InputCommand::new("right".to_string(), moves_needed));
    } else if moves_needed < 0 {
        commands.push(InputCommand::new("left".to_string(), -moves_needed));
    }
    
    commands.push(InputCommand::new("drop".to_string(), 1));
    Ok(commands)
}

fn get_actual_position_after_rotation(piece: &str, spawn_x: i32, rotation: u8) -> i32 {
    match piece {
        "I" => match rotation % 2 {
            0 => spawn_x,
            _ => spawn_x + 2,
        },
        "S" | "Z" => match rotation % 2 {
            0 => spawn_x,
            _ => spawn_x + 1,
        },
        "T" => match rotation % 4 {
            0 => spawn_x,
            1 => spawn_x + 1,
            2 => spawn_x,
            3 => spawn_x,
            _ => spawn_x,
        },
        "J" => match rotation % 4 {
            0 => spawn_x,
            1 => spawn_x + 1,
            2 => spawn_x,
            3 => spawn_x,
            _ => spawn_x,
        },
        "L" => match rotation % 4 {
            0 => spawn_x,
            1 => spawn_x + 1,
            2 => spawn_x,
            3 => spawn_x,
            _ => spawn_x,
        },
        _ => spawn_x,
    }
}

// Global game state
static mut GAME_BOARD: Option<TetrisBoard> = None;

// Core functions
#[pyfunction]
fn initialize_game_board() -> PyResult<()> {
    unsafe {
        GAME_BOARD = Some(TetrisBoard::new());
    }
    Ok(())
}

#[pyfunction]
fn update_game_pieces(current: Option<String>, held: Option<String>, next: Vec<Option<String>>) -> PyResult<()> {
    unsafe {
        if let Some(board) = &mut GAME_BOARD {
            board.update_pieces(current, held, next);
        }
    }
    Ok(())
}

#[pyfunction]
fn get_optimal_move_with_lookahead(current_piece: String) -> PyResult<Option<(Move, Vec<InputCommand>)>> {
    unsafe {
        if let Some(board) = &GAME_BOARD {
            let next_piece = if !board.next_pieces.is_empty() {
                board.next_pieces.get(0).and_then(|p| p.as_ref())
            } else {
                None
            };
            
            let mut best_move: Option<Move> = None;
            let mut best_score = f64::NEG_INFINITY;
            
            for rotation in 0..4 {
                for x in 0..BOARD_WIDTH as i32 {
                    if board.is_valid_position(&current_piece, x, 0, rotation) {
                        let y = board.get_drop_position(&current_piece, x, rotation);
                        
                        let mut board_after_current = board.clone();
                        if board_after_current.place_piece(&current_piece, x, y, rotation) {
                            let mut score = evaluate_board(&board_after_current);
                            
                            // 2-piece lookahead
                            if let Some(next_piece_str) = next_piece {
                                let mut best_next_score = f64::NEG_INFINITY;
                                
                                for next_rotation in 0..4 {
                                    for next_x in 0..BOARD_WIDTH as i32 {
                                        if board_after_current.is_valid_position(next_piece_str, next_x, 0, next_rotation) {
                                            let next_y = board_after_current.get_drop_position(next_piece_str, next_x, next_rotation);
                                            
                                            let mut board_after_both = board_after_current.clone();
                                            if board_after_both.place_piece(next_piece_str, next_x, next_y, next_rotation) {
                                                let next_score = evaluate_board(&board_after_both);
                                                if next_score > best_next_score {
                                                    best_next_score = next_score;
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                score = best_next_score;
                            }
                            
                            if score > best_score {
                                best_score = score;
                                best_move = Some(Move {
                                    piece: current_piece.clone(),
                                    x,
                                    y,
                                    rotation,
                                    score,
                                });
                            }
                        }
                    }
                }
            }
            
            if let Some(best_move) = best_move {
                let commands = calculate_input_commands(&current_piece, &best_move)?;
                return Ok(Some((best_move, commands)));
            }
        }
    }
    Ok(None)
}

#[pyfunction]
fn execute_move_on_board(mv: &Move) -> PyResult<bool> {
    unsafe {
        if let Some(board) = &mut GAME_BOARD {
            return Ok(board.place_piece(&mv.piece, mv.x, mv.y, mv.rotation));
        }
    }
    Ok(false)
}

#[pyfunction]
fn advance_piece_queue() -> PyResult<Option<String>> {
    unsafe {
        if let Some(board) = &mut GAME_BOARD {
            if !board.next_pieces.is_empty() {
                return Ok(board.next_pieces.remove(0));
            }
        }
    }
    Ok(None)
}

#[pyfunction]
fn add_piece_to_queue(piece: Option<String>) -> PyResult<()> {
    unsafe {
        if let Some(board) = &mut GAME_BOARD {
            board.next_pieces.push(piece);
        }
    }
    Ok(())
}

#[pymodule]
fn tetris_bot_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(initialize_game_board, m)?)?;
    m.add_function(wrap_pyfunction!(update_game_pieces, m)?)?;
    m.add_function(wrap_pyfunction!(get_optimal_move_with_lookahead, m)?)?;  // Updated name
    m.add_function(wrap_pyfunction!(execute_move_on_board, m)?)?;
    m.add_function(wrap_pyfunction!(advance_piece_queue, m)?)?;
    m.add_function(wrap_pyfunction!(add_piece_to_queue, m)?)?;
    
    m.add_class::<TetrisBoard>()?;
    m.add_class::<Move>()?;
    m.add_class::<InputCommand>()?;
    Ok(())
}