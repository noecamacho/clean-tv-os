"""Aurora 06 (lume blue): a low-res smooth base, upscaled, plus crisp vector stars, against a
1080p reference rendered directly. Usage: aurora06.py OUTDIR"""
import math, random, struct, sys, zlib

TAU = math.pi * 2
PETROL = (15, 23, 30)
ACCENT = (77, 179, 255)
CYAN = (70, 215, 255)
INDIGO = (70, 90, 230)
SEED = 21


def smooth(a, b, x):
    t = min(1.0, max(0.0, (x - a) / (b - a)))
    return t * t * (3 - 2 * t)


def mix(c1, c2, t):
    return tuple(a + (b - a) * t for a, b in zip(c1, c2))


def column_noise(seed, fa, fb):
    rnd = random.Random(seed)
    pa, pb = rnd.random() * TAU, rnd.random() * TAU
    return lambda u: math.sin(TAU * fa * u + pa) * 0.6 + math.sin(TAU * fb * u + pb) * 0.4


LAYERS = [
    dict(base=0.52, amp=0.08, f1=1.3, f2=2.7, rf1=22, rf2=41, ray=0.5, h=0.18, gain=0.95, low=ACCENT, high=INDIGO),
    dict(base=0.42, amp=0.07, f1=1.7, f2=3.3, rf1=18, rf2=37, ray=0.55, h=0.13, gain=0.65, low=CYAN, high=ACCENT),
]


def stars(w, h, n=420):
    rnd = random.Random(SEED)
    out = []
    for _ in range(n):
        out.append((rnd.randrange(w), rnd.randrange(int(h * 0.7)), 0.25 + rnd.random() * 0.75))
    return out


def base(w, h, edge_px):
    """The smooth aurora, rows of (r, g, b) floats 0-255, without stars."""
    cols = []
    for i, L in enumerate(LAYERS):
        wave = column_noise(SEED * 10 + i, L["f1"], L["f2"])
        rays = column_noise(SEED * 100 + i, L["rf1"], L["rf2"])
        edge = [(L["base"] + L["amp"] * wave(x / w)) * h for x in range(w)]
        ray = [1.0 - L["ray"] + L["ray"] * (0.5 + 0.5 * rays(x / w)) for x in range(w)]
        cols.append((edge, ray))
    rows = []
    for y in range(h):
        v = y / h
        sky = mix((5, 9, 18), PETROL, smooth(0.0, 1.0, v))
        top_fade = smooth(0.0, 0.12, v)
        dark = 1.0 - 0.6 * smooth(0.60, 1.0, v)
        row = []
        for x in range(w):
            r, g, b = sky
            for (edge, ray), L in zip(cols, LAYERS):
                d = edge[x] - y
                if d < -edge_px:
                    continue
                inten = math.exp(-d / (L["h"] * h)) if d >= 0 else 1.0 + d / edge_px
                inten *= ray[x] * L["gain"] * top_fade
                t = min(1.0, max(0.0, d / (L["h"] * h * 2.2)))
                c = mix(L["low"], L["high"], t)
                r += c[0] * inten
                g += c[1] * inten
                b += c[2] * inten
            row.append((255 * (1 - math.exp(-r / 150)) * dark,
                        255 * (1 - math.exp(-g / 150)) * dark,
                        255 * (1 - math.exp(-b / 150)) * dark))
        rows.append(row)
    return rows


def quantize(rows):
    return [[(min(255, int(r)), min(255, int(g)), min(255, int(b))) for r, g, b in row] for row in rows]


def upscale(rows, w, h):
    sh, sw = len(rows), len(rows[0])
    xs = []
    for x in range(w):
        fx = (x + 0.5) * sw / w - 0.5
        x0 = max(0, min(sw - 1, int(math.floor(fx))))
        x1 = min(sw - 1, x0 + 1)
        xs.append((x0, x1, min(1.0, max(0.0, fx - math.floor(fx)))))
    out = []
    for y in range(h):
        fy = (y + 0.5) * sh / h - 0.5
        y0 = max(0, min(sh - 1, int(math.floor(fy))))
        y1 = min(sh - 1, y0 + 1)
        ty = min(1.0, max(0.0, fy - math.floor(fy)))
        a, b = rows[y0], rows[y1]
        row = []
        for x0, x1, tx in xs:
            p00, p10, p01, p11 = a[x0], a[x1], b[x0], b[x1]
            row.append(tuple(
                (p00[c] * (1 - tx) + p10[c] * tx) * (1 - ty) + (p01[c] * (1 - tx) + p11[c] * tx) * ty
                for c in range(3)))
        out.append(row)
    return out


def add_stars(rows, w, h, scale=1):
    for x, y, k in stars(w, h):
        size = 2 if k > 0.8 else 1
        for dy in range(size):
            for dx in range(size):
                if x + dx < w and y + dy < h:
                    r, g, b = rows[y + dy][x + dx]
                    rows[y + dy][x + dx] = (min(255, r + 200 * k), min(255, g + 210 * k), min(255, b + 235 * k))
    return rows


def png(path, rows):
    h, w = len(rows), len(rows[0])
    raw = b"".join(b"\x00" + bytes(min(255, int(c)) for px in row for c in px) for row in rows)

    def chunk(tag, data):
        body = tag + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body))

    open(path, "wb").write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
                           + chunk(b"IDAT", zlib.compress(raw, 6)) + chunk(b"IEND", b""))


def crop(rows, x, y, w, h, zoom=2):
    out = []
    for yy in range(y, y + h):
        row = []
        for xx in range(x, x + w):
            row.extend([rows[yy][xx]] * zoom)
        out.extend([row] * zoom)
    return out


def mae(a, b):
    total = 0.0
    n = 0
    for ra, rb in zip(a, b):
        for pa, pb in zip(ra, rb):
            total += abs(pa[0] - pb[0]) + abs(pa[1] - pb[1]) + abs(pa[2] - pb[2])
            n += 3
    return total / n


out = sys.argv[1]
W, H = 1920, 1080
ref = add_stars(base(W, H, 8.0), W, H)  # the ideal: rendered at full resolution
results = {}
for tw, th in [(640, 360), (480, 270)]:
    # The texture's edge is as soft as 3 of its own texels, so it reproduces faithfully.
    tex = quantize(base(tw, th, 3.0))
    raw = b"".join(bytes(px) for row in tex for px in row)
    size_raw, size_deflate = len(raw), len(zlib.compress(raw, 9))
    final = add_stars(upscale(tex, W, H), W, H)
    err = mae(final, ref)
    results[(tw, th)] = (size_raw, size_deflate, err)
    png(f"{out}/06-final-{tw}x{th}-1080p.png", final)
    png(f"{out}/06-crop-{tw}x{th}.png", crop(final, 700, 380, 480, 270))
png(f"{out}/06-ref-1080p.png", ref)
png(f"{out}/06-crop-ref.png", crop(ref, 700, 380, 480, 270))
for k, (raw, defl, err) in results.items():
    print(f"{k[0]}x{k[1]}: raw RGB {raw/1024:.0f} KB, deflate {defl/1024:.0f} KB, mean abs error vs ref {err:.2f}/255")
