from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter


ROOT = Path(__file__).resolve().parents[1]
ICONS_DIR = ROOT / "src-tauri" / "icons"
MASTER_SIZE = 1024


def vertical_gradient(size: int, top: tuple[int, int, int], bottom: tuple[int, int, int]) -> Image.Image:
    image = Image.new("RGBA", (size, size))
    draw = ImageDraw.Draw(image)
    for y in range(size):
      ratio = y / max(size - 1, 1)
      color = tuple(int(top[index] * (1 - ratio) + bottom[index] * ratio) for index in range(3))
      draw.line((0, y, size, y), fill=color + (255,))
    return image


def draw_capsule(draw: ImageDraw.ImageDraw, p1: tuple[float, float], p2: tuple[float, float], width: int, fill: tuple[int, int, int, int]) -> None:
    draw.line([p1, p2], fill=fill, width=width)
    radius = width // 2
    draw.ellipse((p1[0] - radius, p1[1] - radius, p1[0] + radius, p1[1] + radius), fill=fill)
    draw.ellipse((p2[0] - radius, p2[1] - radius, p2[0] + radius, p2[1] + radius), fill=fill)


def make_master_icon() -> Image.Image:
    image = vertical_gradient(MASTER_SIZE, (18, 20, 28), (8, 10, 16))
    draw = ImageDraw.Draw(image)

    margin = 64
    radius = 220
    draw.rounded_rectangle(
      (margin, margin, MASTER_SIZE - margin, MASTER_SIZE - margin),
      radius=radius,
      fill=(17, 19, 27, 255)
    )

    glow = Image.new("RGBA", (MASTER_SIZE, MASTER_SIZE), (0, 0, 0, 0))
    glow_draw = ImageDraw.Draw(glow)
    glow_draw.ellipse((120, 130, 700, 700), fill=(16, 195, 215, 72))
    glow_draw.ellipse((330, 290, 930, 910), fill=(255, 180, 46, 62))
    glow = glow.filter(ImageFilter.GaussianBlur(70))
    image.alpha_composite(glow)

    surface = Image.new("RGBA", (MASTER_SIZE, MASTER_SIZE), (0, 0, 0, 0))
    surface_draw = ImageDraw.Draw(surface)
    surface_draw.rounded_rectangle(
      (margin, margin, MASTER_SIZE - margin, MASTER_SIZE - margin),
      radius=radius,
      fill=(21, 24, 33, 240),
      outline=(255, 255, 255, 18),
      width=3
    )
    image.alpha_composite(surface)

    mark = Image.new("RGBA", (MASTER_SIZE, MASTER_SIZE), (0, 0, 0, 0))
    mark_draw = ImageDraw.Draw(mark)

    amber = (255, 188, 59, 255)
    cyan = (39, 203, 228, 255)
    dark = (14, 16, 23, 255)
    soft = (245, 248, 255, 255)

    draw_capsule(mark_draw, (310, 300), (708, 698), 172, amber)
    draw_capsule(mark_draw, (708, 300), (310, 698), 172, cyan)

    mark_draw.ellipse((400, 400, 624, 624), fill=dark)
    mark_draw.rounded_rectangle((454, 336, 570, 688), radius=58, fill=soft)
    mark_draw.rounded_rectangle((336, 454, 688, 570), radius=58, fill=soft)
    mark_draw.rounded_rectangle((482, 200, 542, 306), radius=28, fill=amber)
    mark_draw.rounded_rectangle((482, 718, 542, 824), radius=28, fill=cyan)

    shadow = mark.filter(ImageFilter.GaussianBlur(20))
    shadow = shadow.point(lambda value: value * 0.38)
    image.alpha_composite(shadow, (0, 16))
    image.alpha_composite(mark)

    return image


def save_png(image: Image.Image, size: int, path: Path) -> None:
    resized = image.resize((size, size), Image.Resampling.LANCZOS)
    resized.save(path, format="PNG")


def main() -> None:
    ICONS_DIR.mkdir(parents=True, exist_ok=True)
    master = make_master_icon()

    png_sizes = {
      "32x32.png": 32,
      "128x128.png": 128,
      "128x128@2x.png": 256,
      "icon.png": 512,
      "Square30x30Logo.png": 30,
      "Square44x44Logo.png": 44,
      "Square71x71Logo.png": 71,
      "Square89x89Logo.png": 89,
      "Square107x107Logo.png": 107,
      "Square142x142Logo.png": 142,
      "Square150x150Logo.png": 150,
      "Square284x284Logo.png": 284,
      "Square310x310Logo.png": 310,
      "StoreLogo.png": 50
    }

    for filename, size in png_sizes.items():
      save_png(master, size, ICONS_DIR / filename)

    ico_master = master.resize((256, 256), Image.Resampling.LANCZOS)
    ico_master.save(
      ICONS_DIR / "icon.ico",
      format="ICO",
      sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    )

    icns_master = master.resize((1024, 1024), Image.Resampling.LANCZOS)
    icns_master.save(ICONS_DIR / "icon.icns", format="ICNS")


if __name__ == "__main__":
    main()
