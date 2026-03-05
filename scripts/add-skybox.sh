#!/bin/bash
set -euo pipefail

if [ $# -lt 1 ]; then
    echo "Usage: $0 <poly_haven_hdri_name>"
    echo "Example: $0 belfast_sunset"
    exit 1
fi

NAME="$1"
FACE_SIZE=512
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
ASSET_DIR="$PROJECT_DIR/assets/textures"
TMPDIR=$(mktemp -d)

trap 'rm -rf "$TMPDIR"' EXIT

HDR_URL="https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/1k/${NAME}_1k.hdr"
HDR_FILE="$TMPDIR/${NAME}_1k.hdr"
STACKED_PNG="$TMPDIR/${NAME}_cubemap.png"

echo "==> Downloading $HDR_URL"
curl -fSL -o "$HDR_FILE" "$HDR_URL"

echo "==> Converting equirect HDR → stacked cubemap PNG (${FACE_SIZE}px faces)"
uv run --with='py360convert,opencv-python-headless,Pillow,numpy' python3 - "$HDR_FILE" "$STACKED_PNG" "$FACE_SIZE" <<'PYEOF'
import sys
import numpy as np
import cv2
from PIL import Image
import py360convert

hdr_path, out_path, face_size = sys.argv[1], sys.argv[2], int(sys.argv[3])

equirect = cv2.imread(hdr_path, cv2.IMREAD_UNCHANGED)
equirect = cv2.cvtColor(equirect, cv2.COLOR_BGR2RGB).astype(np.float32)

# py360convert face order: F R B L U D
# wgpu cubemap face order: +X(R) -X(L) +Y(U) -Y(D) +Z(F) -Z(B)
cube_faces = py360convert.e2c(equirect, face_w=face_size, cube_format='list')
wgpu_order = [1, 3, 4, 5, 0, 2]  # R L U D F B
ordered = [cube_faces[i] for i in wgpu_order]

# Reinhard tonemap HDR → LDR
stacked = np.vstack(ordered)
stacked = stacked / (1.0 + stacked)
stacked = np.clip(stacked * 255, 0, 255).astype(np.uint8)

Image.fromarray(stacked).save(out_path)
print(f"Wrote {out_path} ({face_size}x{face_size * 6})")
PYEOF

echo "==> Creating ASTC KTX2 (native, no transcoding)"
ASTC_FILE="$TMPDIR/${NAME}_cubemap_astc4x4.ktx2"
ktx create --format ASTC_4x4_SRGB_BLOCK \
    --assign-oetf srgb --assign-primaries bt709 \
    "$STACKED_PNG" "$ASTC_FILE"

echo "==> Creating ETC2 KTX2"
ETC2_FILE="$TMPDIR/${NAME}_cubemap_etc2.ktx2"
toktx --2d --target_type RGB --t2 --encode etc1s --clevel 5 --qlevel 255 \
    --assign_oetf srgb "$ETC2_FILE" "$STACKED_PNG"

echo "==> Moving outputs to $ASSET_DIR"
cp "$STACKED_PNG" "$ASSET_DIR/${NAME}_cubemap.png"
cp "$ASTC_FILE" "$ASSET_DIR/${NAME}_cubemap_astc4x4.ktx2"
cp "$ETC2_FILE" "$ASSET_DIR/${NAME}_cubemap_etc2.ktx2"

echo "==> Done! Assets for '$NAME':"
ls -lh "$ASSET_DIR/${NAME}_cubemap"*
