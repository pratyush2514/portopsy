#!/bin/bash
set -e

PROJECT_DIR="/mnt/c/Users/Pratyush/Downloads/portopsy"
cd "$PROJECT_DIR"

export PATH="$PROJECT_DIR/.vhs-bin:/home/pratyush/.cargo/bin:/home/pratyush/go/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
export LD_LIBRARY_PATH="$PROJECT_DIR/.vhs-bin"

echo "=== Building & Installing portopsy ==="
cargo build --release
# Copy binary directly to system PATH so VHS subshells find it instantly
sudo install -m 755 target/release/portopsy /usr/local/bin/portopsy

echo "vhs:      $(command -v vhs)"
echo "ffmpeg:   $(command -v ffmpeg)"
echo "portopsy: $(command -v portopsy)"

rm -f demo.gif

# Clear port 3000 if previously occupied
fuser -k 3000/tcp 2>/dev/null || true
sleep 0.5

echo "=== Starting background server on :3000 ==="
python3 -m http.server 3000 > /tmp/portopsy-vhs-server.log 2>&1 &
DEMO_PID=$!

trap 'echo "Cleaning up PID \(DEMO_PID..."; kill\)DEMO_PID 2>/dev/null || true' EXIT INT TERM

sleep 1

echo "=== Recording VHS Tape ==="
set +e
vhs demo.tape
RESULT=$?
set -e

echo "VHS_EXIT=$RESULT"
ls -lh demo.gif
