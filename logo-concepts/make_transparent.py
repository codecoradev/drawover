from PIL import Image
import numpy as np

# Load A8
img = Image.open('A8_solid_pencil_mark_orange.png').convert('RGBA')
arr = np.array(img)

# A8 has pure white background. Make near-white pixels transparent.
r, g, b = arr[:,:,0], arr[:,:,1], arr[:,:,2]
white_mask = (r > 240) & (g > 240) & (b > 240)

arr[:,:,3] = np.where(white_mask, 0, 255)
result = Image.fromarray(arr)
result.save('A8_drawover_logo_transparent.png')
pct = white_mask.sum() / white_mask.size * 100
print(f"Transparent: {pct:.0f}% | Opaque: {100-pct:.0f}%")
print(f"Size: {result.size}")
