"""Generate app icon as ICO and PNG from SVG source."""
import struct
import zlib
import os

SIZE = 256

def create_png(width, height, pixels):
    """Create a minimal PNG from raw RGBA pixels."""
    def chunk(chunk_type, data):
        c = chunk_type + data
        crc = struct.pack(">I", zlib.crc32(c) & 0xffffffff)
        return struct.pack(">I", len(data)) + c + crc

    raw = b""
    for y in range(height):
        raw += b"\x00"  # filter none
        for x in range(width):
            idx = (y * width + x) * 4
            raw += pixels[idx:idx+4]

    sig = b"\x89PNG\r\n\x1a\n"
    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return sig + chunk(b"IHDR", ihdr) + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b"")

def generate_icon():
    """Generate AudioMaster Pro icon - purple waveform on dark background."""
    pixels = bytearray(SIZE * SIZE * 4)

    cx, cy = SIZE // 2, SIZE // 2
    radius = SIZE // 2 - 4

    for y in range(SIZE):
        for x in range(SIZE):
            dx, dy = x - cx, y - cy
            dist = (dx*dx + dy*dy) ** 0.5
            idx = (y * SIZE + x) * 4

            if dist <= radius:
                # Dark background with slight gradient
                t = dist / radius
                r = int(5 + t * 10)
                g = int(4 + t * 8)
                b = int(15 + t * 20)
                a = 255

                # Draw waveform bars in the center
                bar_width = 6
                num_bars = 11
                total_w = num_bars * bar_width + (num_bars - 1) * 3
                start_x = cx - total_w // 2

                for bar_i in range(num_bars):
                    bx = start_x + bar_i * (bar_width + 3)
                    if bx <= x < bx + bar_width:
                        # Waveform height pattern (symmetric)
                        heights = [0.3, 0.5, 0.7, 0.85, 1.0, 0.9, 1.0, 0.85, 0.7, 0.5, 0.3]
                        max_h = int(radius * 0.55 * heights[bar_i])
                        bar_top = cy - max_h
                        bar_bot = cy + max_h

                        if bar_top <= y <= bar_bot:
                            # Purple gradient
                            pct = abs(y - cy) / max(max_h, 1)
                            r = int(168 + (200 - 168) * pct)
                            g = int(85 + (50 - 85) * pct)
                            b = int(247 + (255 - 247) * pct)
                            a = 255

                            # Rounded bar ends (circular corners)
                            corner_r = bar_width // 2
                            cx_bar = bx + bar_width // 2
                            if y - bar_top < corner_r:
                                dy = corner_r - (y - bar_top)
                                dx = abs(x - cx_bar)
                                if dx * dx + dy * dy > corner_r * corner_r:
                                    a = 0
                            elif bar_bot - y < corner_r:
                                dy = corner_r - (bar_bot - y)
                                dx = abs(x - cx_bar)
                                if dx * dx + dy * dy > corner_r * corner_r:
                                    a = 0

                pixels[idx] = r
                pixels[idx+1] = g
                pixels[idx+2] = b
                pixels[idx+3] = a
            else:
                pixels[idx:idx+4] = b"\x00\x00\x00\x00"

    return bytes(pixels)

def create_ico(png_data, sizes):
    """Create ICO file from PNG data."""
    # ICO header
    header = struct.pack("<HHH", 0, 1, len(sizes))

    # We'll create PNG entries for each size
    entries = []
    png_chunks = []
    offset = 6 + len(sizes) * 16  # header + entries

    for size in sizes:
        ico_size = 0 if size >= 256 else size
        entry = struct.pack("<BBBBHHIH", 
            ico_size, ico_size, 0, 0, 1, 32, len(png_data), offset)
        entries.append(entry)
        png_chunks.append(png_data)
        offset += len(png_data)

    return header + b"".join(entries) + b"".join(png_chunks)

if __name__ == "__main__":
    out_dir = os.path.dirname(os.path.abspath(__file__))
    
    print("Generating icon pixels...")
    pixels = generate_icon()
    
    print("Creating PNG...")
    png_256 = create_png(SIZE, SIZE, pixels)
    
    png_path = os.path.join(out_dir, "icon.png")
    with open(png_path, "wb") as f:
        f.write(png_256)
    print(f"Saved {png_path}")
    
    # Create ICO with multiple sizes
    ico_path = os.path.join(out_dir, "icon.ico")
    ico_data = create_ico(png_256, [16, 32, 48, 64, 128, 256])
    with open(ico_path, "wb") as f:
        f.write(ico_data)
    print(f"Saved {ico_path}")
    
    print("Done!")
