import os, requests, time, sys
from dotenv import load_dotenv
load_dotenv('/opt/data/.env')

BEARER = os.environ.get('MODAL_API_KEY', '')
headers = {'Authorization': f'Bearer {BEARER}'}

# Use FLUX.2 Klein 4B
BASE = 'https://komandancrypto--flux2-klein-4b-fastapi-fastapi-app.modal.run'

OUT_DIR = '/opt/data/profiles/cmo/work/drawover/logo-concepts'
os.makedirs(OUT_DIR, exist_ok=True)

concepts = [
    {
        'name': 'A_pencil_arc_dark',
        'prompt': 'A minimalist macOS app icon for a screen annotation tool called DrawOver. '
                  'A stylized white pencil tip drawing a smooth curved arc line over a rounded square. '
                  'The stroke transitions from warm orange (#EF593F) to soft amber. '
                  'Deep charcoal-near-black background (#1A1A2E) with subtle grid texture. '
                  'Apple macOS Big Sur app icon aesthetic, flat design with subtle depth gradient, '
                  'soft top-down studio light. No text, no letters, no typography, no watermark. '
                  'Clean vector style, 8K, professional app icon, centered composition.',
    },
    {
        'name': 'B_lens_doodles_orange',
        'prompt': 'A minimalist macOS app icon for a screen annotation tool. '
                  'A bold geometric magnifying glass where the glass area contains colorful hand-drawn doodle strokes — '
                  'a red squiggly line, a yellow circle, a green underline. '
                  'The handle is sleek matte black. '
                  'Background: solid vibrant warm orange (#EF593F) filling the entire frame. '
                  'Apple macOS app icon aesthetic, rounded squircle, flat design with subtle drop shadows, '
                  'playful but professional. No text, no letters, no typography, no watermark. '
                  'Clean geometric shapes, centered.',
    },
    {
        'name': 'C_glowing_marker_night',
        'prompt': 'A minimalist macOS app icon for a screen annotation tool. '
                  'A glowing digital marker pen hovering diagonally over a dark glass surface, '
                  'leaving a bright luminous orange trail. Small sparkle particles emit from the marker tip. '
                  'The marker: sleek matte white body with silver clip, angled at 45 degrees. '
                  'Background: deep dark navy gradient (#0F0F1A to #1A1A2E), subtle reflection. '
                  'Apple macOS Big Sur app icon, 3D render with soft realistic lighting, '
                  'dramatic rim light, warm orange glow from the trail. '
                  'No text, no letters, no typography, no watermark. 8K, centered.',
    },
]

# Health check
print("Health check...", flush=True)
try:
    t0 = time.time()
    r = requests.get(f'{BASE}/healthz', timeout=300, headers=headers)
    print(f"  Status: {r.status_code} | Time: {time.time()-t0:.1f}s", flush=True)
except Exception as e:
    print(f"  Health error: {e}", flush=True)

# Generate each concept
for i, concept in enumerate(concepts):
    name = concept['name']
    prompt = concept['prompt']
    out_path = os.path.join(OUT_DIR, f'{name}.png')

    print(f"\n[{i+1}/3] Generating: {name}", flush=True)
    print(f"  Prompt: {prompt[:80]}...", flush=True)

    try:
        t0 = time.time()
        r = requests.post(
            f'{BASE}/generate_image',
            files={
                'prompt': (None, prompt),
                'width': (None, '1024'),
                'height': (None, '1024'),
            },
            timeout=600,
            headers=headers,
        )
        elapsed = time.time() - t0

        if r.status_code == 200 and 'image' in r.headers.get('content-type', ''):
            with open(out_path, 'wb') as f:
                f.write(r.content)
            size_kb = os.path.getsize(out_path) / 1024
            print(f"  ✅ Saved: {out_path} ({size_kb:.0f}KB) | {elapsed:.1f}s", flush=True)
        else:
            print(f"  ❌ HTTP {r.status_code} | {elapsed:.1f}s", flush=True)
            print(f"  Response: {r.text[:300]}", flush=True)
    except Exception as e:
        print(f"  ❌ Error: {e}", flush=True)

print("\n=== DONE ===", flush=True)
for f in sorted(os.listdir(OUT_DIR)):
    p = os.path.join(OUT_DIR, f)
    print(f"  {f}: {os.path.getsize(p)/1024:.0f}KB")
