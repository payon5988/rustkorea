"""Generate PWA icons (pure stdlib: struct + zlib) matching the SVG gear mark."""
import math
import struct
import zlib

BG = (222, 91, 48, 255)
WHITE = (255, 255, 255, 255)


def gear_polygon(s, scale=1.0):
    cx = cy = s / 2.0
    gear_r = s * 0.35 * scale
    inner_r = gear_r * 0.7
    outer_r = gear_r
    teeth = 8
    tooth_w = math.pi * 2.0 / teeth * 0.35
    pts = []
    for i in range(teeth):
        a = math.pi * 2.0 * i / teeth
        a1, a2 = a - tooth_w, a + tooth_w
        quad = [
            (cx + inner_r * math.cos(a1), cy + inner_r * math.sin(a1)),
            (cx + outer_r * math.cos(a1), cy + outer_r * math.sin(a1)),
            (cx + outer_r * math.cos(a2), cy + outer_r * math.sin(a2)),
            (cx + inner_r * math.cos(a2), cy + inner_r * math.sin(a2)),
        ]
        if i == 0:
            pts.append(quad[0])
        pts.extend(quad[1:])
    return pts


def scanline_fill(poly, width, height):
    """Even-odd scanline fill. Returns list of bytearray rows (1 inside)."""
    edges = []
    n = len(poly)
    for i in range(n):
        x1, y1 = poly[i]
        x2, y2 = poly[(i + 1) % n]
        if y1 == y2:
            continue
        edges.append((x1, y1, x2, y2))
    mask = [bytearray(width) for _ in range(height)]
    for y in range(height):
        yc = y + 0.5
        xs = []
        for x1, y1, x2, y2 in edges:
            if (y1 <= yc < y2) or (y2 <= yc < y1):
                t = (yc - y1) / (y2 - y1)
                xs.append(x1 + t * (x2 - x1))
        xs.sort()
        for i in range(0, len(xs) - 1, 2):
            for x in range(max(0, math.floor(xs[i])), min(width, math.ceil(xs[i + 1]))):
                if xs[i] <= x + 0.5 <= xs[i + 1]:
                    mask[y][x] = 1
    return mask


def render(size, maskable=False, supersample=2):
    w = h = size * supersample
    poly = gear_polygon(w, scale=0.72 if maskable else 1.0)
    mask = scanline_fill(poly, w, h)
    cx = cy = w / 2.0
    hole_r = w * 0.12 * (0.72 if maskable else 1.0)
    if not maskable:
        corner_r = w * 0.15
    px = bytearray(w * h * 4)
    for y in range(h):
        row = mask[y]
        for x in range(w):
            inside = row[x]
            dx = min(x, w - 1 - x)
            dy = min(y, h - 1 - y)
            if not maskable and (dx < corner_r and dy < corner_r):
                if (corner_r - dx - 0.5) ** 2 + (corner_r - dy - 0.5) ** 2 > corner_r ** 2:
                    inside = 0
                    # outside rounded corner -> transparent
                    continue
            if inside:
                ox, oy = x - cx, y - cy
                if ox * ox + oy * oy <= hole_r * hole_r:
                    col = BG
                else:
                    col = WHITE
            else:
                col = BG
            o = (y * w + x) * 4
            px[o:o + 4] = bytes(col)
    # box downsample
    out = bytearray(size * size * 4)
    for y in range(size):
        for x in range(size):
            r = g = b = a = 0
            for dy in range(supersample):
                for dx in range(supersample):
                    o = (((y * supersample + dy) * w) + (x * supersample + dx)) * 4
                    # transparent corner pixels are zeros (already skipped -> zeros)
                    r += px[o]
                    g += px[o + 1]
                    b += px[o + 2]
                    a += px[o + 3]
            n = supersample * supersample
            # un-premultiply-ish: average; transparent corners average toward 0 alpha
            o = (y * size + x) * 4
            # detect fully transparent block
            if a == 0:
                out[o:o + 4] = b'\x00\x00\x00\x00'
            else:
                # composite over nothing: keep averaged color, averaged alpha
                out[o:o + 4] = bytes((r // n, g // n, b // n, a // n if not maskable else 255))
    return bytes(out)


def write_png(path, size, rgba):
    def chunk(typ, data):
        c = struct.pack('>I', len(data)) + typ + data
        return c + struct.pack('>I', zlib.crc32(typ + data) & 0xFFFFFFFF)

    ihdr = struct.pack('>IIBBBBB', size, size, 8, 6, 0, 0, 0)
    raw = b''.join(b'\x00' + rgba[y * size * 4:(y + 1) * size * 4] for y in range(size))
    png = (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', ihdr)
           + chunk(b'IDAT', zlib.compress(raw, 9)) + chunk(b'IEND', b''))
    with open(path, 'wb') as f:
        f.write(png)
    print(path, len(png), "bytes")


if __name__ == '__main__':
    import os
    os.makedirs('public/icons', exist_ok=True)
    write_png('public/icons/icon-192.png', 192, render(192))
    write_png('public/icons/icon-512.png', 512, render(512))
    write_png('public/icons/maskable-512.png', 512, render(512, maskable=True))
    write_png('public/icons/apple-touch-icon.png', 180, render(180))
