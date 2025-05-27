import time
from PIL import ImageGrab, Image
import cv2
import numpy as np
import keyboard

# Add this import at the top after building the Rust module
try:
    import tetris_bot_rust
    USE_RUST = True
    print("Rust module available, using Rust implementation")
except ImportError:
    USE_RUST = False
    print("Rust module not available, using Python implementation")

class TetrisController:
    def __init__(self):
        # Define key mappings - testing with k and ; for rotation
        self.keys = {
            'left': 'a',
            'right': 'd', 
            'rotate_cw': 'k',           # Changed from 'left' to 'k'
            'rotate_ccw': ';',          # Changed from 'right' to ';'
            'rotate_180': 'o',          # Changed from 'up' to 'o'
            'soft_drop': 's',
            'hard_drop': 'space',
            'hold': 'w',
        }
        
        # Timing settings - reduced for faster gameplay
        self.move_delay = 0.04       # Reduced from 0.08
        self.rotation_delay = 0.08   # Reduced from 0.15
        self.drop_delay = 0.15       # Reduced from 0.3
        
    def execute_move(self, move):
        """Execute a move returned by the Rust AI"""
        if not move:
            return False
            
        try:
            print(f"Executing move for {move.piece}: x={move.x}, rotation={move.rotation}")
            
            # Handle rotations
            if move.rotation > 0:
                for i in range(int(move.rotation)):
                    keyboard.press(self.keys['rotate_cw'])
                    keyboard.release(self.keys['rotate_cw'])
                    time.sleep(self.rotation_delay)
            
            # Handle horizontal movement
            current_x = 4  # Assume starting position
            moves_needed = int(move.x) - current_x
            
            if moves_needed != 0:
                direction = 'right' if moves_needed > 0 else 'left'
                for i in range(abs(moves_needed)):
                    keyboard.press_and_release(self.keys[direction])
                    time.sleep(self.move_delay)
            
            # Drop the piece
            keyboard.press_and_release(self.keys['hard_drop'])
            time.sleep(self.drop_delay)
            
            return True
            
        except Exception as e:
            print(f"Error executing move: {e}")
            return False
    
    def hold_piece(self):
        """Hold the current piece"""
        try:
            keyboard.press_and_release(self.keys['hold'])
            time.sleep(self.drop_delay)
            return True
        except Exception as e:
            print(f"Error holding piece: {e}")
            return False

    def test_rotation_keys(self):
        """Test rotation keys using k, ;, and o bindings"""
        print("Testing rotation keys with k, ;, and o bindings...")
        
        print("Testing k - clockwise rotation:")
        keyboard.press_and_release('k')
        time.sleep(1.5)
        
        print("Testing ; - counter-clockwise rotation:")
        keyboard.press_and_release(';')
        time.sleep(1.5)
        
        print("Testing o - 180-degree flip:")
        keyboard.press_and_release('o')
        time.sleep(1.5)
        
        print("Rotation key test complete")
        
    def execute_commands(self, commands):
        """Execute a list of input commands from Rust"""
        if not commands:
            return False
            
        try:
            print(f"Executing {len(commands)} commands:")
            
            for i, command in enumerate(commands):
                print(f"  Command {i+1}: {command.action} x{command.count}")
                
                # Map Rust command names to Python key names
                action_map = {
                    'left': 'left',
                    'right': 'right',
                    'rotate_cw': 'rotate_cw',
                    'rotate_ccw': 'rotate_ccw',
                    'rotate_180': 'rotate_180',
                    'drop': 'hard_drop',
                    'hold': 'hold'
                }
                
                if command.action in action_map:
                    key_name = action_map[command.action]
                    key = self.keys[key_name]
                    
                    for j in range(command.count):
                        print(f"    Pressing key: {key} ({j+1}/{command.count})")
                        keyboard.press_and_release(key)
                        
                        # Use appropriate delay based on action type
                        if command.action in ['rotate_cw', 'rotate_ccw', 'rotate_180']:
                            time.sleep(self.rotation_delay)
                        elif command.action in ['left', 'right']:
                            time.sleep(self.move_delay)
                        elif command.action == 'drop':
                            time.sleep(self.drop_delay)
                else:
                    print(f"    Unknown action: {command.action}")
            
            print("  All commands executed!")
            return True
            
        except Exception as e:
            print(f"  Error executing commands: {e}")
            return False

    def debug_single_rotation(self):
        """Debug a single rotation to see what happens"""
        print("Debug: Testing single clockwise rotation...")
        
        try:
            # Method 2: Key name (this worked)
            print("Method 2: 'left' key name")
            keyboard.press_and_release('k')
            time.sleep(2)
            
            # Method 3: Explicit press/release (this worked)
            print("Method 3: Explicit press/release")
            keyboard.press('k')
            time.sleep(0.1)
            keyboard.release('k')
            time.sleep(2)
            
        except Exception as e:
            print(f"Error in debug rotation: {e}")

def find_next_by_template_matching(search_area, template_path, threshold=0.8):
    """Find the "next" text using template matching"""
    x, y, width, height = search_area
    
    screenshot = ImageGrab.grab(bbox=(x, y, x + width, y + height))
    screenshot_cv = cv2.cvtColor(np.array(screenshot), cv2.COLOR_RGB2BGR)
    
    try:
        template = cv2.imread(template_path, cv2.IMREAD_COLOR)
        if template is None:
            return None
    except Exception:
        return None
    
    template_height, template_width = template.shape[:2]
    result = cv2.matchTemplate(screenshot_cv, template, cv2.TM_CCOEFF_NORMED)
    min_val, max_val, min_loc, max_loc = cv2.minMaxLoc(result)
    
    if max_val >= threshold:
        match_x = x + max_loc[0]
        match_y = y + max_loc[1]
        return (match_x, match_y, template_width, template_height)
    else:
        return None

def find_next_optimized(search_area, template_path, threshold=0.7):
    """Optimized template matching with 50% resolution for speed"""
    x, y, width, height = search_area
    
    screenshot = ImageGrab.grab(bbox=(x, y, x + width, y + height))
    
    # Resize to 50% for faster processing
    small_screenshot = screenshot.resize((width//2, height//2))
    screenshot_cv = cv2.cvtColor(np.array(small_screenshot), cv2.COLOR_RGB2BGR)
    
    try:
        template = cv2.imread(template_path, cv2.IMREAD_COLOR)
        if template is None:
            return None
        
        # Also resize template
        template_height, template_width = template.shape[:2]
        small_template = cv2.resize(template, (template_width//2, template_height//2))
        
    except Exception:
        return None
    
    result = cv2.matchTemplate(screenshot_cv, small_template, cv2.TM_CCOEFF_NORMED)
    min_val, max_val, min_loc, max_loc = cv2.minMaxLoc(result)
    
    if max_val >= threshold:
        # Scale back up to original coordinates
        match_x = x + (max_loc[0] * 2)
        match_y = y + (max_loc[1] * 2)
        return (match_x, match_y, template_width, template_height)
    else:
        return None

def identify_piece_by_color_fast(pixel_color, tetris_colors):
    """Fast piece identification"""
    if hasattr(pixel_color[0], 'item'):
        pixel_color = tuple(int(c.item()) for c in pixel_color)
    else:
        pixel_color = tuple(int(c) for c in pixel_color)
    
    best_match = None
    min_distance = float('inf')
    
    for piece_name, piece_color in tetris_colors.items():
        distance = sum((a - b) ** 2 for a, b in zip(pixel_color, piece_color))
        
        if distance < min_distance:
            min_distance = distance
            best_match = piece_name
    
    return best_match

def get_game_pieces_ultra_optimized(search_area, template_path, piece_pixel_offsets, current_piece_offset):
    """Ultra-optimized: single screenshot for all piece detection"""
    
    next_coords = find_next_optimized(search_area, template_path, threshold=0.7)
    
    if not next_coords:
        return None, []
    
    text_x, text_y, text_width, text_height = next_coords
    
    # Calculate all pixel positions
    current_pixel_x = text_x + current_piece_offset[0]
    current_pixel_y = text_y + current_piece_offset[1]
    
    piece_positions = []
    for x_offset, y_offset in piece_pixel_offsets:
        pixel_x = text_x + x_offset
        pixel_y = text_y + text_height + y_offset
        piece_positions.append((pixel_x, pixel_y))
    
    # Find bounding box for all pixels
    all_x_coords = [current_pixel_x] + [pos[0] for pos in piece_positions]
    all_y_coords = [current_pixel_y] + [pos[1] for pos in piece_positions]
    
    min_x = min(all_x_coords)
    max_x = max(all_x_coords)
    min_y = min(all_y_coords)
    max_y = max(all_y_coords)
    
    # Take ONE screenshot covering all needed pixels
    screenshot = ImageGrab.grab(bbox=(min_x, min_y, max_x + 1, max_y + 1))
    
    # Color definitions
    tetris_colors = {
        'I': (49, 178, 130),
        'O': (179, 153, 49),
        'T': (207, 60, 193),
        'S': (131, 179, 50),
        'Z': (179, 52, 59),
        'J': (78, 61, 164),
        'L': (180, 99, 50)
    }
    
    # Extract current piece color
    current_rel_x = current_pixel_x - min_x
    current_rel_y = current_pixel_y - min_y
    current_color = screenshot.getpixel((current_rel_x, current_rel_y))
    current_piece = identify_piece_by_color_fast(current_color, tetris_colors)
    
    # Extract next pieces colors
    next_pieces = []
    for pixel_x, pixel_y in piece_positions:
        rel_x = pixel_x - min_x
        rel_y = pixel_y - min_y
        color = screenshot.getpixel((rel_x, rel_y))
        piece = identify_piece_by_color_fast(color, tetris_colors)
        next_pieces.append(piece)
    
    return current_piece, next_pieces

def main_game_loop():
    """Simplified main game loop - Rust does all the thinking"""
    # Configuration
    SEARCH_AREA = (0, 0, 1920, 1080)
    TEMPLATE_PATH = "assets/next_template.png"
    CURRENT_PIECE_OFFSET = (-225, -60)
    
    PIECE_PIXEL_OFFSETS = [
        (104, 60), (104, 165), (104, 270), (104, 375), (104, 480)
    ]
    
    if not USE_RUST:
        print("Rust module not available - cannot run game loop")
        return
    
    # Initialize components
    controller = TetrisController()
    
    # Initialize persistent board state in Rust
    tetris_bot_rust.initialize_game_board()
    
    print("TETRIS BOT ACTIVE - Starting in 3 seconds...")
    time.sleep(3)
    
    try:
        next_coords = find_next_optimized(SEARCH_AREA, TEMPLATE_PATH, threshold=0.7)
        
        if not next_coords:
            print("Could not find 'next' text!")
            return
        
        print("Starting game loop...")
        moves_executed = 0
        
        while True:
            try:
                # 1. Detect pieces (Python)
                current_piece, next_pieces = get_game_pieces_ultra_optimized(
                    SEARCH_AREA, TEMPLATE_PATH, PIECE_PIXEL_OFFSETS, CURRENT_PIECE_OFFSET
                )
                
                if current_piece:
                    print(f"\nMove #{moves_executed + 1}: Detected {current_piece}")
                    
                    # 2. Update game state (Rust)
                    tetris_bot_rust.update_game_pieces(current_piece, None, next_pieces)
                    
                    # 3. Get optimal move and input commands (Rust)
                    result = tetris_bot_rust.get_optimal_move_with_inputs(current_piece)
                    
                    if result:
                        best_move, commands = result
                        print(f"AI Decision: x={best_move.x}, rot={best_move.rotation}, score={best_move.score:.1f}")
                        
                        # 4. Execute commands (Python)
                        if controller.execute_commands(commands):
                            # 5. Update virtual board state (Rust)
                            tetris_bot_rust.execute_move_on_board(best_move)
                            moves_executed += 1
                            time.sleep(0.5)  # Reduced from 1.0 - faster between pieces
                    else:
                        print("No valid move found!")
                else:
                    print("Waiting for piece detection...")
                
                time.sleep(0.1)  # Reduced from 0.2 - faster detection loop
                
            except KeyboardInterrupt:
                print(f"\nStopping bot... Total moves executed: {moves_executed}")
                break
            except Exception as e:
                print(f"Error in game loop: {e}")
                time.sleep(1)
        
    except Exception as e:
        print(f"Fatal error: {e}")

if __name__ == "__main__":
    import sys
    
    if len(sys.argv) > 1 and sys.argv[1] == "test_rotation":
        # Test rotation keys specifically
        controller = TetrisController()
        controller.test_rotation_keys()
        
    elif len(sys.argv) > 1 and sys.argv[1] == "debug_rotation":
        # Debug single rotation with multiple methods
        controller = TetrisController()
        controller.debug_single_rotation()
        
    elif len(sys.argv) > 1 and sys.argv[1] == "play":
        main_game_loop()
        
    else:
        # Normal detection test
        print("=== TETRIS BOT - DETECTION TEST ===")
        print("Run with:")
        print("  'python main.py test_rotation' - test rotation keys")
        print("  'python main.py debug_rotation' - debug rotation methods")
        print("  'python main.py play' - start playing")
        
        SEARCH_AREA = (0, 0, 1920, 1080)
        TEMPLATE_PATH = "assets/next_template.png"
        CURRENT_PIECE_OFFSET = (-225, -60)
        
        PIECE_PIXEL_OFFSETS = [
            (104, 60), (104, 165), (104, 270), (104, 375), (104, 480)
        ]   
        
        # Create Tetris board
        if USE_RUST:
            board = tetris_bot_rust.create_board()
            print("Rust Tetris engine initialized")
        
        # Detect pieces
        current_piece, next_pieces = get_game_pieces_ultra_optimized(
            SEARCH_AREA, TEMPLATE_PATH, PIECE_PIXEL_OFFSETS, CURRENT_PIECE_OFFSET
        )
        
        print(f"\nDetection results:")
        print(f"  Current piece: {current_piece if current_piece else 'Not detected'}")
        print(f"  Next pieces:")
        for i, piece in enumerate(next_pieces[:5], 1):
            print(f"    {i}: {piece if piece else 'Not detected'}")
        
        # Calculate move
        if USE_RUST and current_piece:
            try:
                board.update_pieces(current_piece, None, next_pieces)
                best_move = tetris_bot_rust.calculate_best_move(board, current_piece)
                
                if best_move:
                    print(f"\nRust AI Decision:")
                    print(f"  Best move: Place {best_move.piece} at x={best_move.x}, rotation={best_move.rotation}")
                    print(f"  Score: {best_move.score:.1f}")
                else:
                    print("  No valid move found!")
                    
            except Exception as e:
                print(f"  Error with Rust engine: {e}")