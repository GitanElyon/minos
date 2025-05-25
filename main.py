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
        # Calculate distance using sum of squared differences (no sqrt needed)
        distance = sum((a - b) ** 2 for a, b in zip(pixel_color, piece_color))
        
        if distance < min_distance:
            min_distance = distance
            best_match = piece_name
    
    return best_match

def get_game_pieces_ultra_optimized(search_area, template_path, piece_pixel_offsets, current_piece_offset):
    """Ultra-optimized: single screenshot for all piece detection"""
    
    # Find "next" text coordinates
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

if __name__ == "__main__":
    # Configuration
    SEARCH_AREA = (0, 0, 1920, 1080)
    TEMPLATE_PATH = "assets/next_template.png"
    CURRENT_PIECE_OFFSET = (-225, -60)
    
    PIECE_PIXEL_OFFSETS = [
        (104, 60), (104, 165), (104, 270), (104, 375), (104, 480)
    ]   
    
    print("=== TETRIS BOT - ULTRA-OPTIMIZED DETECTION ===")
    
    print("\n  Timing breakdown:")
    start_time = time.perf_counter()
    
    # Get all pieces with optimized method
    current_piece, next_pieces = get_game_pieces_ultra_optimized(
        SEARCH_AREA, TEMPLATE_PATH, PIECE_PIXEL_OFFSETS, CURRENT_PIECE_OFFSET
    )
    
    total_time = (time.perf_counter() - start_time) * 1000
    print(f"  Total detection time: {total_time:.1f}ms")
    
    # Display results
    print(f"\n  Detection results:")
    print(f"  Current piece: {current_piece if current_piece else 'Not detected'}")
    print(f"  Next pieces:")
    for i, piece in enumerate(next_pieces[:5], 1):
        print(f"    {i}: {piece if piece else 'Not detected'}")