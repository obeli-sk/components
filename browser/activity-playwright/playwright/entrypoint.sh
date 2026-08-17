#!/usr/bin/env bash
set -euo pipefail

XVFB_PID=""
X11VNC_PID=""

cleanup() {
  if [ -n "$X11VNC_PID" ] && kill -0 "$X11VNC_PID" 2>/dev/null; then
    kill "$X11VNC_PID" 2>/dev/null || true
    wait "$X11VNC_PID" 2>/dev/null || true
  fi
  if [ -n "$XVFB_PID" ] && kill -0 "$XVFB_PID" 2>/dev/null; then
    kill "$XVFB_PID" 2>/dev/null || true
    wait "$XVFB_PID" 2>/dev/null || true
  fi
}

trap cleanup EXIT INT TERM

if [ "${HEADED:-}" = "true" ]; then
  export DISPLAY=:99
  Xvfb :99 -screen 0 1600x1200x24 -ac +extension RANDR &
  XVFB_PID=$!

  for _ in $(seq 1 50); do
    if [ -S /tmp/.X11-unix/X99 ]; then
      break
    fi
    sleep 0.1
  done

  if [ ! -S /tmp/.X11-unix/X99 ]; then
    echo '"Xvfb did not create DISPLAY :99"'
    exit 1
  fi

  x11vnc \
    -display :99 \
    -listen 0.0.0.0 \
    -rfbport 5900 \
    -forever \
    -shared \
    -nopw \
    -xkb \
    >/tmp/x11vnc.log 2>&1 &
  X11VNC_PID=$!

  echo "Headed mode enabled with Xvfb on :99 and VNC on port 5900" >&2
fi

exec node /app/server.js "$@"
