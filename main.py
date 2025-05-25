import pyautogui
from PIL import ImageGrab, Image
import time
import cv2
import numpy as np

def hex_to_rgb(hex_color):
    """Convert hex color to RGB tuple"""
    hex_color = hex_color.lstrip('#')
    return tuple(int(hex_color[i:i+2], 16) for i in (0, 2, 4))

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

def is_black_or_dark(color, threshold=30):
    """Check if a color is black or very dark"""
    if hasattr(color[0], 'item'):
        color = tuple(int(c.item()) for c in color)
    else:
        color = tuple(int(c) for c in color)
    
    return all(c < threshold for c in color)

def identify_piece_by_color(pixel_color, tetris_colors):
    """Identify which Tetris piece matches the given pixel color"""
    if hasattr(pixel_color[0], 'item'):
        pixel_color = tuple(int(c.item()) for c in pixel_color)
    else:
        pixel_color = tuple(int(c) for c in pixel_color)
    
    best_match = None
    min_distance = float('inf')
    
    for piece_name, piece_color in tetris_colors.items():
        piece_color = tuple(int(c) for c in piece_color)
        distance = np.sqrt(sum((a - b) ** 2 for a, b in zip(pixel_color, piece_color)))
        
        if distance < min_distance:
            min_distance = distance
            best_match = piece_name
    
    return best_match if min_distance <= 80 else None

def find_piece_color_in_section(img_array, section_start, section_end, piece_width):
    """Find a non-black color within a piece section"""
    center_x = img_array.shape[1] // 2 if img_array.shape[1] > 1 else 0
    section_height = section_end - section_start
    
    sample_points = [
        section_start + section_height // 4,
        section_start + section_height // 2,
        section_start + 3 * section_height // 4,
        section_start + section_height // 8,
        section_start + 7 * section_height // 8
    ]
    
    for y in sample_points:
        if 0 <= y < img_array.shape[0]:
            color = tuple(img_array[y, center_x])
            if not is_black_or_dark(color):
                return color, y
    
    # Fallback to center point
    fallback_y = section_start + section_height // 2
    if 0 <= fallback_y < img_array.shape[0]:
        fallback_color = tuple(img_array[fallback_y, center_x])
        return fallback_color, fallback_y
    
    return None, None

def detect_next_pieces(piece_area, num_pieces=5):
    """Detect the next pieces in the queue"""
    if not piece_area:
        return []
    
    piece_x, piece_y, piece_width, piece_height = piece_area
    screenshot = ImageGrab.grab(bbox=(piece_x, piece_y, piece_x + piece_width, piece_y + piece_height))
    img_array = np.array(screenshot)
    
    tetris_colors = {
        'I': (49, 178, 130),
        'O': (179, 153, 49),
        'T': (207, 60, 193),
        'S': (131, 179, 50),
        'Z': (179, 52, 59),
        'J': (78, 61, 164),
        'L': (180, 99, 50)
    }
    
    section_height = piece_height // num_pieces
    detected_pieces = []
    
    for i in range(num_pieces):
        section_start = i * section_height
        section_end = min((i + 1) * section_height, img_array.shape[0])
        
        color, sample_y = find_piece_color_in_section(img_array, section_start, section_end, piece_width)
        
        if color is not None:
            detected_piece = identify_piece_by_color(color, tetris_colors)
            detected_pieces.append(detected_piece)
        else:
            detected_pieces.append(None)
    
    return detected_pieces

def detect_next_pieces_optimized(next_coords, piece_pixel_offsets):
    """
    Detect next pieces by checking specific pixel coordinates
    
    Args:
        next_coords: (x, y, width, height) of the found "next" text
        piece_pixel_offsets: List of (x_offset, y_offset) for each piece position
    
    Returns:
        List of detected pieces
    """
    if not next_coords:
        return []
    
    text_x, text_y, text_width, text_height = next_coords
    
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
    
    for i, (x_offset, y_offset) in enumerate(piece_pixel_offsets):
        # Calculate absolute pixel position
        pixel_x = text_x + x_offset
        pixel_y = text_y + text_height + y_offset
        
        # Capture just this one pixel
        screenshot = ImageGrab.grab(bbox=(pixel_x, pixel_y, pixel_x + 1, pixel_y + 1))
        pixel_color = screenshot.getpixel((0, 0))
        
        # Convert to tuple if needed
        if isinstance(pixel_color, tuple):
            color = pixel_color
        else:
            color = (pixel_color, pixel_color, pixel_color)  # Handle grayscale
        
        # Identify piece by color
        if not is_black_or_dark(color):
            detected_piece = identify_piece_by_color(color, tetris_colors)
            detected_pieces.append(detected_piece)
        else:
            detected_pieces.append(None)
    
    return detected_pieces

def detect_current_piece(next_coords, current_piece_offset):
    """Detect the current piece"""
    if not next_coords:
        return None
    
    text_x, text_y, text_width, text_height = next_coords
    
    current_x = text_x + current_piece_offset[0]
    current_y = text_y + current_piece_offset[1] 
    current_width = current_piece_offset[2]
    current_height = current_piece_offset[3]
    
    screenshot = ImageGrab.grab(bbox=(current_x, current_y, 
                                    current_x + current_width, 
                                    current_y + current_height))
    img_array = np.array(screenshot)
    
    tetris_colors = {
        'I': (49, 178, 130),
        'O': (179, 153, 49),
        'T': (207, 60, 193),
        'S': (131, 179, 50),
        'Z': (179, 52, 59),
        'J': (78, 61, 164),
        'L': (180, 99, 50)
    }
    
    # Sample multiple points to find piece color
    for y_frac in [0.3, 0.5, 0.7]:
        for x_frac in [0.3, 0.5, 0.7]:
            sample_x = int(current_width * x_frac)
            sample_y = int(current_height * y_frac)
            
            if 0 <= sample_y < img_array.shape[0] and 0 <= sample_x < img_array.shape[1]:
                color = tuple(img_array[sample_y, sample_x])
                
                if not is_black_or_dark(color):
                    return identify_piece_by_color(color, tetris_colors)
    
    return None

def get_game_pieces(search_area, template_path, piece_offset, current_piece_offset):
    """Get current piece and next 5 pieces"""
    next_coords = find_next_by_template_matching(search_area, template_path, threshold=0.7)
    
    if not next_coords:
        return None, []
    
    # Get current piece
    current_piece = detect_current_piece(next_coords, current_piece_offset)
    
    # Get next pieces
    text_x, text_y, text_width, text_height = next_coords
    adjusted_y_offset = piece_offset[1] - 2
    piece_x = text_x + piece_offset[0] - (piece_offset[2] - text_width) // 2
    piece_y = text_y + text_height + adjusted_y_offset
    piece_width = piece_offset[2]
    piece_height = piece_offset[3] + 2
    
    piece_area = (piece_x, piece_y, piece_width, piece_height)
    next_pieces = detect_next_pieces(piece_area)
    
    return current_piece, next_pieces

def get_game_pieces_optimized(search_area, template_path, piece_pixel_offsets, current_piece_offset):
    """
    Get current piece and next 5 pieces using optimized pixel checking
    
    Args:
        search_area: Area to search for "next" text
        template_path: Path to next template image
        piece_pixel_offsets: List of 5 (x_offset, y_offset) tuples for piece positions
        current_piece_offset: (x_offset, y_offset, width, height) for current piece
    """
    next_coords = find_next_by_template_matching(search_area, template_path, threshold=0.7)
    
    if not next_coords:
        return None, []
    
    # Get current piece
    current_piece = detect_current_piece(next_coords, current_piece_offset)
    
    # Get next pieces using optimized method
    next_pieces = detect_next_pieces_optimized(next_coords, piece_pixel_offsets)
    
    return current_piece, next_pieces

if __name__ == "__main__":
    # Configuration
    SEARCH_AREA = (0, 0, 1920, 1080)
    TEMPLATE_PATH = "assets/next_template.png"
    CURRENT_PIECE_OFFSET = (-225, -60, 1, 1)
    
    # Define exact pixel offsets from "next" text for each of the 5 pieces
    # Adjust these coordinates to point to the center of each piece preview
    PIECE_PIXEL_OFFSETS = [
        (104, 60),   # Piece 1 offset from "next" text
        (104, 165),  # Piece 2 offset from "next" text  
        (104, 270),  # Piece 3 offset from "next" text
        (104, 375),  # Piece 4 offset from "next" text
        (104, 480)   # Piece 5 offset from "next" text
    ]   
    
    print("=== TETRIS BOT - OPTIMIZED PIECE DETECTION ===")
    
    # Start timing
    start_time = time.perf_counter()
    
    # Get current and next pieces
    current_piece, next_pieces = get_game_pieces_optimized(
        SEARCH_AREA, TEMPLATE_PATH, PIECE_PIXEL_OFFSETS, CURRENT_PIECE_OFFSET
    )
    
    # End timing
    end_time = time.perf_counter()
    detection_time = (end_time - start_time) * 1000  # Convert to milliseconds
    
    # Display results
    print(f"\nCurrent piece: {current_piece if current_piece else 'Not detected'}")
    
    print(f"\nNext pieces:")
    for i, piece in enumerate(next_pieces[:5], 1):
        print(f"  {i}: {piece if piece else 'Not detected'}")
    
    print(f"\n  Detection completed in {detection_time:.1f}ms")