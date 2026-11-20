#!/usr/bin/env bash
# Screenshots of the running dashboard for the demo video. Usage: demo/shoot.sh <name> "<query string>"
CH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
out="$(dirname "$0")/shots/$1.png"; rm -f "$out"
("$CH" --headless=new --use-angle=swiftshader --enable-unsafe-swiftshader --hide-scrollbars --window-size=1920,1080 --virtual-time-budget=6000 --user-data-dir=/private/tmp/claude-501/chrome-shot --screenshot="$out" "http://localhost:3000/?static=1&$2" >/dev/null 2>&1 &)
for i in $(seq 1 40); do [ -s "$out" ] && break; perl -e 'select(undef,undef,undef,1)'; done
pkill -f "chrome-shot" >/dev/null 2>&1; [ -s "$out" ] && echo "ok $1" || echo "FAILED $1"
