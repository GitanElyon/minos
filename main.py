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
        # Define key mappings
        self.keys = {
            'left': 'a',
            'right': 'd', 
            'rotate_cw': ';',
            'rotate_ccw': 'k',
            'rotate_180': 'o',
            'soft_drop': 's',
            'hard_drop': 'space',
            'hold': 'w',
        }
        
        # Timing settings
        self.move_delay = 0.04
        self.rotation_delay = 0.08
        self.drop_delay = 0.15
        
    def execute_commands(self, commands):
        """Execute a list of input commands from Rust"""
        if not commands:
            return False
            
        try:
            for command in commands:
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
                    
                    for _ in range(command.count):
                        keyboard.press_and_release(key)
                        
                        if command.action in ['rotate_cw', 'rotate_ccw', 'rotate_180']:
                            time.sleep(self.rotation_delay)
                        elif command.action in ['left', 'right']:
                            time.sleep(self.move_delay)
                        elif command.action == 'drop':
                            time.sleep(self.drop_delay)
            
            return True
            
        except Exception as e:
            print(f"Error executing commands: {e}")
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

def identify_piece_by_color_robust(pixel_color, tetris_colors, min_confidence=50):
    """More robust piece identification with confidence threshold"""
    if hasattr(pixel_color[0], 'item'):
        pixel_color = tuple(int(c.item()) for c in pixel_color)
    else:
        pixel_color = tuple(int(c) for c in pixel_color)
    
    best_match = None
    min_distance = float('inf')
    second_best_distance = float('inf')
    
    for piece_name, piece_color in tetris_colors.items():
        # Use Euclidean distance in RGB space
        distance = ((pixel_color[0] - piece_color[0])**2 + 
                   (pixel_color[1] - piece_color[1])**2 + 
                   (pixel_color[2] - piece_color[2])**2) ** 0.5
        
        if distance < min_distance:
            second_best_distance = min_distance
            min_distance = distance
            best_match = piece_name
        elif distance < second_best_distance:
            second_best_distance = distance
    
    # Check confidence - if two colors are too similar, return None
    confidence = second_best_distance - min_distance
    if confidence < min_confidence:
        print(f"Low confidence detection: {pixel_color} -> {best_match} (conf: {confidence:.1f})")
        return None
    
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

def scan_single_piece(template_path, search_area, piece_pixel_offset):
    """Scan just one piece position"""
    next_coords = find_next_optimized(search_area, template_path, threshold=0.7)
    
    if not next_coords:
        return None
    
    text_x, text_y, text_width, text_height = next_coords
    
    # Calculate pixel position for the piece
    pixel_x = text_x + piece_pixel_offset[0]
    pixel_y = text_y + text_height + piece_pixel_offset[1]
    
    # Take screenshot of just this one pixel
    screenshot = ImageGrab.grab(bbox=(pixel_x, pixel_y, pixel_x + 1, pixel_y + 1))
    
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
    
    color = screenshot.getpixel((0, 0))
    piece = identify_piece_by_color_fast(color, tetris_colors)
    
    return piece

def main_game_loop():
    """Main game loop with optimized piece tracking"""
    # Configuration
    SEARCH_AREA = (0, 0, 1920, 1080)
    TEMPLATE_PATH = "assets/next_template.png"
    CURRENT_PIECE_OFFSET = (-225, -60)
    
    PIECE_PIXEL_OFFSETS = [
        (104, 60), (104, 165), (104, 270), (104, 375), (104, 480)
    ]
    
    # Last piece in queue (5th position) - this is what we'll scan each turn
    LAST_QUEUE_POSITION = PIECE_PIXEL_OFFSETS[4]  # (104, 480)
    
    if not USE_RUST:
        print("Rust module not available - cannot run game loop")
        return
    
    # Initialize components
    controller = TetrisController()
    tetris_bot_rust.initialize_game_board()
    
    print("TETRIS BOT ACTIVE - Starting in 3 seconds...")
    time.sleep(3)
    
    try:
        # INITIAL SCAN: Get current piece + full queue
        print("Scanning initial pieces...")
        current_piece, next_pieces = get_game_pieces_ultra_optimized(
            SEARCH_AREA, TEMPLATE_PATH, PIECE_PIXEL_OFFSETS, CURRENT_PIECE_OFFSET
        )
        
        if not current_piece or not next_pieces:
            print("Could not detect initial pieces!")
            return
        
        print(f"Initial current piece: {current_piece}")
        print(f"Initial queue: {next_pieces}")
        
        # Initialize the game board with all detected pieces
        tetris_bot_rust.update_game_pieces(current_piece, None, next_pieces)
        
        print("Starting game loop...")
        moves_executed = 0
        current_playing_piece = current_piece
        
        while True:
            try:
                if current_playing_piece:
                    print(f"Playing piece: {current_playing_piece}")
                    
                    # Get optimal move
                    result = tetris_bot_rust.get_optimal_move_with_lookahead(current_playing_piece)
                    
                    if result:
                        best_move, commands = result
                        
                        # Execute commands
                        if controller.execute_commands(commands):
                            tetris_bot_rust.execute_move_on_board(best_move)
                            moves_executed += 1
                            
                            # ADVANCE QUEUE: Get next piece from queue
                            current_playing_piece = tetris_bot_rust.advance_piece_queue()
                            
                            # SCAN NEW PIECE: Only scan the last queue position
                            print("Scanning for new queue piece...")
                            new_piece = scan_single_piece(
                                TEMPLATE_PATH, SEARCH_AREA, LAST_QUEUE_POSITION
                            )
                            
                            if new_piece:
                                tetris_bot_rust.add_piece_to_queue(new_piece)
                                print(f"Added new piece to queue: {new_piece}")
                            else:
                                print("Warning: Could not detect new piece for queue")
                            
                            time.sleep(0.1)
                else:
                    print("No current piece available!")
                    break
                
                time.sleep(0.1)
                
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
    
    if len(sys.argv) > 1 and sys.argv[1] == "play":
        main_game_loop()
    else:
        print("=== TETRIS BOT ===")
        print("Run with 'python main.py play' to start playing")