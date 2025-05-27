use pyo3::prelude::*;

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

    fn print_board_simple(&self) {
        println!("Current board:");
        for y in 0..BOARD_HEIGHT {
            print!("  ");
            for x in 0..BOARD_WIDTH {
                let cell = self.grid[y][x];
                if cell == 0 {
                    print!(".");
                } else {
                    print!("#");
                }
            }
            println!();
        }
        println!();
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

fn get_spawn_position(piece: &str) -> (i32, i32) {
    match piece {
        "I" => (3, 0),
        "O" => (4, 0),
        _ => (3, 0),
    }
}

// Evaluation functions
fn evaluate_board(board: &TetrisBoard) -> f64 {
    let line_clear_weight = 760.666;
    let hole_weight = -35.0;
    let bumpiness_weight = -18.0;
    let height_weight = -51.0;
    
    let lines_cleared = count_complete_lines(board) as f64;
    let holes = count_holes(board) as f64;
    let bumpiness = get_bumpiness(board) as f64;
    let aggregate_height = get_aggregate_height(board) as f64;
    
    lines_cleared * line_clear_weight +
    holes * hole_weight +
    bumpiness * bumpiness_weight +
    aggregate_height * height_weight
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

// Global board state
static mut GAME_BOARD: Option<TetrisBoard> = None;

// Python functions
#[pyfunction]
fn initialize_game_board() -> PyResult<()> {
    unsafe {
        GAME_BOARD = Some(TetrisBoard::new());
    }
    Ok(())
}

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

#[pyfunction]
fn calculate_best_move_persistent(
    piece: Option<String>,
) -> PyResult<Option<Move>> {
    if let Some(piece_type) = piece {
        unsafe {
            if let Some(board) = &mut GAME_BOARD {
                let mut best_move = None;
                let mut best_score = f64::NEG_INFINITY;
                
                for rotation in 0..4 {
                    for x in 0..BOARD_WIDTH as i32 {
                        if board.is_valid_position(&piece_type, x, 0, rotation) {
                            let y = board.get_drop_position(&piece_type, x, rotation);
                            
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
                
                return Ok(best_move);
            }
        }
    }
    
    Ok(None)
}

#[pyfunction]
fn calculate_input_commands(
    piece: &str,
    target_move: &Move
) -> PyResult<Vec<InputCommand>> {
    let mut commands = Vec::new();
    let (spawn_x, _) = get_spawn_position(piece);
    
    println!("Original spawn: x={}, target: x={}, rotation={}", spawn_x, target_move.x, target_move.rotation);
    
    // Handle rotations first
    let rotation = target_move.rotation % 4;
    
    if rotation == 2 {
        commands.push(InputCommand::new("rotate_180".to_string(), 1));
    } else if rotation > 0 {
        commands.push(InputCommand::new("rotate_cw".to_string(), rotation as i32));
    }
    
    // Calculate where the piece actually ends up after rotation
    let actual_position_after_rotation = get_actual_position_after_rotation(piece, spawn_x, rotation);
    
    println!("After rotation, piece is actually at x={}", actual_position_after_rotation);
    
    // Handle movement from actual position to target
    let moves_needed = target_move.x - actual_position_after_rotation;
    
    println!("Need to move {} positions to reach target x={}", moves_needed, target_move.x);
    
    if moves_needed > 0 {
        commands.push(InputCommand::new("right".to_string(), moves_needed));
    } else if moves_needed < 0 {
        commands.push(InputCommand::new("left".to_string(), -moves_needed));
    }
    
    // Drop
    commands.push(InputCommand::new("drop".to_string(), 1));
    
    Ok(commands)
}

// Add this helper function
fn get_actual_position_after_rotation(piece: &str, spawn_x: i32, rotation: u8) -> i32 {
    let result = match piece {
        "I" => {
            match rotation % 2 {
                0 => spawn_x,      // Horizontal I-piece stays at spawn
                _ => spawn_x + 2,  // Vertical I-piece shifts right by 2
            }
        },
        "S" | "Z" => {
            match rotation % 2 {
                0 => spawn_x,      // Horizontal S/Z pieces (3 wide)
                _ => spawn_x + 1,  // Vertical S/Z pieces (2 wide) - shift right by 1
            }
        },
        "T" => {
            match rotation % 4 {
                0 => spawn_x,      // Original T orientation (3 wide)
                1 => spawn_x + 1,  // T rotated 90° right (2 wide) - shift right by 1
                2 => spawn_x,      // T upside down (3 wide) - same as original
                3 => spawn_x,      // T rotated 270° right (2 wide) - peninsula on left, no shift
                _ => spawn_x,
            }
        },
        "J" => {
            match rotation % 4 {
                0 => spawn_x,      // Original J orientation
                1 => spawn_x + 1,  // J rotated 90° right
                2 => spawn_x,      // J upside down
                3 => spawn_x,      // J rotated 270° right - peninsula affects position
                _ => spawn_x,
            }
        },
        "L" => {
            match rotation % 4 {
                0 => spawn_x,      // Original L orientation
                1 => spawn_x + 1,  // L rotated 90° right - try +1 instead of 0
                2 => spawn_x,      // L upside down
                3 => spawn_x,  // L rotated 270° right - peninsula on right
                _ => spawn_x,
            }
        },
        "O" => spawn_x,  // O-piece doesn't shift when rotated
        _ => spawn_x,
    };
    
    // Debug output
    if piece == "L" || piece == "J" || piece == "T" {
        println!("DEBUG: {} rotation {} - spawn_x: {} -> actual_x: {}", 
                 piece, rotation, spawn_x, result);
    }
    
    result
}

#[pyfunction]
fn get_optimal_move_with_inputs(
    piece: Option<String>,
) -> PyResult<Option<(Move, Vec<InputCommand>)>> {
    if let Some(piece_type) = piece {
        if let Some(best_move) = calculate_best_move_persistent(Some(piece_type.clone()))? {
            let commands = calculate_input_commands(&piece_type, &best_move)?;
            return Ok(Some((best_move, commands)));
        }
    }
    
    Ok(None)
}

#[pyfunction]
fn get_optimal_move_with_inputs_debug(
    piece: Option<String>,
) -> PyResult<Option<(Move, Vec<InputCommand>)>> {
    if let Some(piece_type) = piece {
        println!("=== MOVE CALCULATION DEBUG ===");
        println!("Detected piece: {}", piece_type);
        
        // Show current board state
        unsafe {
            if let Some(board) = &GAME_BOARD {
                board.print_board_simple();
            }
        }
        
        // Calculate best move
        if let Some(best_move) = calculate_best_move_persistent(Some(piece_type.clone()))? {
            println!("AI Decision:");
            println!("  Place {} at x={}, rotation={}", 
                     best_move.piece, best_move.x, best_move.rotation);
            println!("  Score: {:.1}", best_move.score);
            
            // Calculate commands
            let commands = calculate_input_commands(&piece_type, &best_move)?;
            
            println!("Input commands:");
            for (i, cmd) in commands.iter().enumerate() {
                println!("  {}: {} x{}", i+1, cmd.action, cmd.count);
            }
            
            println!("=== END DEBUG ===\n");
            
            return Ok(Some((best_move, commands)));
        } else {
            println!("No valid moves found!");
            println!("=== END DEBUG ===\n");
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
fn reset_game_board() -> PyResult<()> {
    unsafe {
        GAME_BOARD = Some(TetrisBoard::new());
    }
    Ok(())
}

#[pymodule]
fn tetris_bot_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(initialize_game_board, m)?)?;
    m.add_function(wrap_pyfunction!(update_game_pieces, m)?)?;
    m.add_function(wrap_pyfunction!(calculate_best_move_persistent, m)?)?;
    m.add_function(wrap_pyfunction!(get_optimal_move_with_inputs, m)?)?;
    m.add_function(wrap_pyfunction!(get_optimal_move_with_inputs_debug, m)?)?;  // Add debug version
    m.add_function(wrap_pyfunction!(execute_move_on_board, m)?)?;
    m.add_function(wrap_pyfunction!(reset_game_board, m)?)?;
    
    m.add_class::<TetrisBoard>()?;
    m.add_class::<Move>()?;
    m.add_class::<InputCommand>()?;
    Ok(())
}