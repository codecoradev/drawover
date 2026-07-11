from PIL import Image
import os

RESAMPLE = Image.Resampling.LANCZOS

src = Image.open('A8_solid_pencil_mark_orange.png').convert('RGBA')
icon_dir = '../src-tauri/icons'
os.makedirs(icon_dir, exist_ok=True)

# Tauri required icon sizes
sizes = {
    '32x32.png': 32,
    '128x128.png': 128,
    '128x128@2x.png': 256,
    'icon.png': 512,
}

for name, size in sizes.items():
    resized = src.resize((size, size), RESAMPLE)
    resized.save(os.path.join(icon_dir, name))
    print(f"  {name} ({size}x{size})")

# Generate .ico (Windows)
ico_sizes = [16, 32, 48, 64, 128, 256]
src.save(os.path.join(icon_dir, 'icon.ico'), format='ICO', sizes=[(s, s) for s in ico_sizes])
print(f"  icon.ico ({len(ico_sizes)} sizes)")

# Generate .icns (macOS) - requires png2icns or manual
# Use pillow if available, otherwise skip
try:
    icns_path = os.path.join(icon_dir, 'icon.icns')
    src.save(icns_path, format='ICNS')
    print(f"  icon.icns")
except Exception as e:
    print(f"  icon.icns SKIPPED ({e})")

# Also save a Square30x30Logo.png and Square44x44Logo.png for Windows tile
for name, sz in [('Square30x30Logo.png', 30), ('Square44x44Logo.png', 44), ('Square71x71Logo.png', 71), ('Square89x89Logo.png', 89), ('Square107x107Logo.png', 107), ('Square142x142Logo.png', 142), ('Square150x150Logo.png', 150), ('Square284x284Logo.png', 284), ('Square310x310Logo.png', 310)]:
    src.resize((sz, sz), RESAMPLE).save(os.path.join(icon_dir, name))

# Copy transparent version as logo asset
logo_dir = '../src/assets'
os.makedirs(logo_dir, exist_ok=True)
transparent = Image.open('A8_drawover_logo_transparent.png')
transparent.save(os.path.join(logo_dir, 'logo.png'))

# Also save full-res to assets
src.save(os.path.join(logo_dir, 'logo-full.png'))

print("\n=== ALL ICONS GENERATED ===")
