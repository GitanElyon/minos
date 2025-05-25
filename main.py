import pyautogui
from PIL import ImageGrab, Image
import time
import cv2
import numpy as np

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

def identify_piece_by_color_fast(pixel_color, tetris_colors):
    """Fast piece identification without unnecessary checks"""
    # Convert numpy types to regular integers if needed
    if hasattr(pixel_color[0], 'item'):
        pixel_color = tuple(int(c.item()) for c in pixel_color)
    else:
        pixel_color = tuple(int(c) for c in pixel_color)
    
    best_match = None
    min_distance = float('inf')
    
    for piece_name, piece_color in tetris_colors.items():
        # Calculate distance using simple sum of squared differences
        distance = sum((a - b) ** 2 for a, b in zip(pixel_color, piece_color))
        
        if distance < min_distance:
            min_distance = distance
            best_match = piece_name
    
    return best_match

def detect_next_pieces_optimized(next_coords, piece_pixel_offsets):
    """Fast detection of next pieces by checking specific pixels"""
    if not next_coords:
        return []
    
    text_x, text_y, text_width, text_height = next_coords
    
    # Pre-defined colors for faster lookup
    tetris_colors = {
        'I': (49, 178, 130),
        'O': (179, 153, 49),
        'T': (207, 60, 193),
        'S': (131, 179, 50),
        'Z': (179, 52, 59),
        'J': (78, 61, 164),
        'L': (180, 99, 50)
    }
    
    detected_pieces = []
    
    for x_offset, y_offset in piece_pixel_offsets:
        # Calculate absolute pixel position
        pixel_x = text_x + x_offset
        pixel_y = text_y + text_height + y_offset
        
        # Capture single pixel
        screenshot = ImageGrab.grab(bbox=(pixel_x, pixel_y, pixel_x + 1, pixel_y + 1))
        pixel_color = screenshot.getpixel((0, 0))
        
        # Identify piece directly
        detected_piece = identify_piece_by_color_fast(pixel_color, tetris_colors)
        detected_pieces.append(detected_piece)
    
    return detected_pieces

def detect_current_piece_fast(next_coords, current_piece_offset):
    """Fast current piece detection"""
    if not next_coords:
        return None
    
    text_x, text_y, text_width, text_height = next_coords
    
    # Calculate position
    current_x = text_x + current_piece_offset[0]
    current_y = text_y + current_piece_offset[1] 
    
    # Capture single pixel
    screenshot = ImageGrab.grab(bbox=(current_x, current_y, current_x + 1, current_y + 1))
    pixel_color = screenshot.getpixel((0, 0))
    
    # Pre-defined colors
    tetris_colors = {
        'I': (49, 178, 130),
        'O': (179, 153, 49),
        'T': (207, 60, 193),
        'S': (131, 179, 50),
        'Z': (179, 52, 59),
        'J': (78, 61, 164),
        'L': (180, 99, 50)
    }
    
    return identify_piece_by_color_fast(pixel_color, tetris_colors)

def get_game_pieces_optimized(search_area, template_path, piece_pixel_offsets, current_piece_offset):
    """Ultra-fast piece detection"""
    next_coords = find_next_by_template_matching(search_area, template_path, threshold=0.7)
    
    if not next_coords:
        return None, []
    
    # Get current piece (single pixel capture)
    current_piece = detect_current_piece_fast(next_coords, current_piece_offset)
    
    # Get next pieces (5 pixel captures)
    next_pieces = detect_next_pieces_optimized(next_coords, piece_pixel_offsets)
    
    return current_piece, next_pieces

if __name__ == "__main__":
    # Configuration
    SEARCH_AREA = (0, 0, 1920, 1080)
    TEMPLATE_PATH = "assets/next_template.png"
    CURRENT_PIECE_OFFSET = (-225, -60)  # Only need x,y now
    
    PIECE_PIXEL_OFFSETS = [
        (104, 60),   # Piece 1
        (104, 165),  # Piece 2  
        (104, 270),  # Piece 3
        (104, 375),  # Piece 4
        (104, 480)   # Piece 5
    ]   
    
    print("=== TETRIS BOT - ULTRA-OPTIMIZED DETECTION ===")
    
    # Start timing
    start_time = time.perf_counter()
    
    # Get current and next pieces
    current_piece, next_pieces = get_game_pieces_optimized(
        SEARCH_AREA, TEMPLATE_PATH, PIECE_PIXEL_OFFSETS, CURRENT_PIECE_OFFSET
    )
    
    # End timing
    end_time = time.perf_counter()
    detection_time = (end_time - start_time) * 1000
    
    # Display results
    print(f"\nCurrent piece: {current_piece if current_piece else 'Not detected'}")
    
    print(f"\nNext pieces:")
    for i, piece in enumerate(next_pieces[:5], 1):
        print(f"  {i}: {piece if piece else 'Not detected'}")
    
    print(f"\n Detection completed in {detection_time:.1f}ms")