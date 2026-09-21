#!/bin/bash
set -e

cargo +nightly build --release
rm -rf target/wasm32-unknown-unknown/release/Payload
mkdir -p target/wasm32-unknown-unknown/release/Payload
cp res/* target/wasm32-unknown-unknown/release/Payload/
cp target/wasm32-unknown-unknown/release/webtoonkr.wasm target/wasm32-unknown-unknown/release/Payload/main.wasm
rm -f target/wasm32-unknown-unknown/release/Payload/.DS_Store

python3 - << 'EOF'
import zipfile, os

with zipfile.ZipFile("package.aix", "w", zipfile.ZIP_DEFLATED) as z:
    added = set()
    base_dir = "target/wasm32-unknown-unknown/release/Payload"
    for f in os.listdir(base_dir):
        path = os.path.join(base_dir, f)
        if not os.path.isfile(path) or f == ".DS_Store":
            continue
        arcname = f"Payload/{f}"
        if arcname not in added:
            z.write(path, arcname)
            added.add(arcname)
        if f.lower() == "icon.png":
            if "Payload/icon.png" not in added:
                z.write(path, "Payload/icon.png")
                added.add("Payload/icon.png")
            if "Payload/Icon.png" not in added:
                z.write(path, "Payload/Icon.png")
                added.add("Payload/Icon.png")
EOF

echo "Build succeeded: package.aix generated"
