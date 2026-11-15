#!/usr/bin/env sh
# Make an 8-bit sounding copy of the original music with ffmpeg's bitcrusher.
# usage: tools/make_8bit_music.sh [bits] [sample_hold]
#   bits:        bit depth to crush to (default 8, lower is grittier)
#   sample_hold: samples held per step, 44100/n is the effective rate (default 4)
set -eu

BITS="${1:-8}"
HOLD="${2:-4}"
DIR="$(cd "$(dirname "$0")/../resources" && pwd)"

for NAME in music_1; do
    ffmpeg -hide_banner -loglevel error -y -i "$DIR/$NAME.ogg" \
        -af "pan=mono|c0=0.5*c0+0.5*c1,highpass=f=90,acrusher=bits=$BITS:mode=lin:aa=0:samples=$HOLD:mix=1,lowpass=f=11000" \
        -ar 44100 -c:a libvorbis -q:a 5 "$DIR/${NAME}_8bit.ogg"
    echo "wrote $DIR/${NAME}_8bit.ogg (bits=$BITS, sample_hold=$HOLD)"
done
