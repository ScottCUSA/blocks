#!/usr/bin/env python3
"""Render the game's original 8-bit music themes to resources/music_<name>.ogg.

Two pulse channels, a triangle bass and a noise drum channel, in the style
of NES-era hardware. Melodies are written as "note:eighths" tokens.

usage: tools/make_chiptune.py [name ...]   (renders every song by default)
"""

import array
import math
import pathlib
import random
import subprocess
import sys
import tempfile
import wave

RATE = 44100
OUT_DIR = pathlib.Path(__file__).resolve().parent.parent / "resources"

NAMES = {
    "C": 0, "C#": 1, "Db": 1, "D": 2, "D#": 3, "Eb": 3, "E": 4, "F": 5, "F#": 6, "Gb": 6,
    "G": 7, "G#": 8, "Ab": 8, "A": 9, "A#": 10, "Bb": 10, "B": 11,
}


def midi(note):
    return NAMES[note[:-1]] + (int(note[-1]) + 1) * 12


def freq_midi(m):
    return 440.0 * 2 ** ((m - 69) / 12)


def phrase(text):
    """'A4:2 C5:1 R:1' -> [('A4', 2), ('C5', 1), ('R', 1)]"""
    return [(note, int(length)) for note, length in (t.split(":") for t in text.split())]


def chord_notes(chord):
    """'Am' -> (root, third, fifth) midi numbers around octave 2"""
    minor = chord.endswith("m")
    root = NAMES[chord[:-1] if minor else chord] + 36
    return root, root + (3 if minor else 4), root + 7


# --- instruments ---------------------------------------------------------------

def steps(value, levels):
    return round(value * (levels - 1)) / (levels - 1)


def pulse(f, n, duty, volume):
    out, phase = [], 0.0
    release = int(RATE * 0.012)
    for i in range(n):
        t = i / RATE
        # gentle vibrato on held notes
        vib = 1 + 0.006 * math.sin(2 * math.pi * 6 * t) * min(1.0, max(0.0, (t - 0.18) * 5))
        phase = (phase + f * vib / RATE) % 1.0
        s = 1.0 if phase < duty else -1.0
        env = steps(0.7 + 0.3 * math.exp(-t * 12), 16) * min(1.0, (n - i) / release)
        out.append(s * env * volume)
    return out


def triangle(f, n, volume):
    out, phase = [], 0.0
    release = int(RATE * 0.008)
    for i in range(n):
        phase = (phase + f / RATE) % 1.0
        s = steps(1 - abs(phase * 2 - 1), 32) * 2 - 1
        out.append(s * volume * min(1.0, (n - i) / release))
    return out


def noise(n, hold, decay, volume, rng):
    out, level = [], 0.0
    for i in range(n):
        if i % hold == 0:
            level = rng.choice((-1.0, 1.0))
        out.append(level * steps(math.exp(-decay * i / RATE), 16) * volume)
    return out


def kick(rng, volume):
    n = int(RATE * 0.12)
    body, phase = [], 0.0
    for i in range(n):
        t = i / RATE
        phase = (phase + (140 * math.exp(-t * 30) + 45) / RATE) % 1.0
        body.append((steps(1 - abs(phase * 2 - 1), 32) * 2 - 1) * math.exp(-t * 18) * 0.9 * volume)
    click = noise(int(RATE * 0.02), 24, 120, 0.4 * volume, rng)
    return [b + (click[i] if i < len(click) else 0) for i, b in enumerate(body)]


def snare(rng, volume):
    return noise(int(RATE * 0.14), 3, 22, 0.45 * volume, rng)


def hat(rng, volume):
    return noise(int(RATE * 0.03), 1, 140, 0.18 * volume, rng)


# bass patterns: offsets from the chord root per eighth, "3" is the chord third
BASS_PATTERNS = {
    "bounce": [0, 12, 7, 12, 0, 12, 7, 12],
    "march": [0, None, 7, None, 0, None, 7, None],
    "arp": [0, 7, 12, "3", 12, 7, 12, 7],
}

# drum patterns: eighth positions for each drum
DRUM_PATTERNS = {
    "rock": {"kick": [0, 4], "snare": [2, 6], "hat": [1, 3, 5, 7]},
    "march": {"kick": [0, 4], "snare": [2, 6, 7], "hat": [1, 3, 5]},
    "minuet": {"kick": [0], "snare": [], "hat": [2, 4]},
    "halftime": {"kick": [0, 5], "snare": [4], "hat": [0, 1, 2, 3, 4, 5, 6, 7]},
}


# --- songs ---------------------------------------------------------------------

SONGS = {
    # driving folk dance, A minor
    "dance": {
        "bpm": 150, "bar": 8, "intro": 2, "bass": "bounce", "drums": "rock", "drum_volume": 1.0,
        "staccato": 1.0, "echo": None,
        "sections": [
            ("A", 0.5, False), ("B", 0.25, False), ("A", 0.5, True), ("C", 0.25, False),
        ],
        "parts": {
            "A": (["Am", "Am", "E", "E", "Am", "Dm", "E", "Am"], phrase("""
                A4:2 C5:1 E5:1 A5:2 G5:1 E5:1
                F5:1 E5:1 D5:1 C5:1 E5:3 R:1
                B4:2 D5:1 E5:1 G#5:2 F5:1 E5:1
                D5:1 C5:1 B4:1 C5:1 D5:3 R:1
                C5:2 E5:1 A5:1 C6:2 B5:1 A5:1
                A5:1 F5:1 D5:1 F5:1 A5:2 G5:1 F5:1
                E5:2 G#5:1 B5:1 E5:1 D5:1 C5:1 B4:1
                A4:4 E4:2 A4:2
            """), None),
            "B": (["F", "G", "Em", "Am", "F", "G", "E", "E"], phrase("""
                A5:3 G5:1 F5:2 E5:2
                D5:3 E5:1 F5:2 D5:2
                G5:3 F5:1 E5:2 B4:2
                C5:3 D5:1 E5:4
                A5:3 B5:1 C6:2 A5:2
                B5:3 A5:1 G5:2 D5:2
                E5:1 F5:1 G#5:1 A5:1 B5:2 G#5:2
                E5:4 B4:1 C5:1 D5:1 E5:1
            """), None),
            "C": (["Dm", "Am", "E", "Am", "Dm", "Am", "E", "E"], phrase("""
                D5:1 F5:1 A5:1 F5:1 D5:1 F5:1 A5:1 D6:1
                C6:1 A5:1 E5:1 A5:1 C6:1 A5:1 E5:1 C5:1
                B4:1 E5:1 G#5:1 B5:1 G#5:1 E5:1 B4:1 G#4:1
                A4:1 C5:1 E5:1 A5:1 E5:2 C5:2
                F5:2 E5:1 D5:1 A5:2 F5:2
                E5:2 D5:1 C5:1 A4:2 C5:2
                D5:2 C5:1 B4:1 G#4:2 B4:2
                E5:6 R:2
            """), None),
        },
    },
    # bright staccato march, C major
    "bounce": {
        "bpm": 168, "bar": 8, "intro": 2, "bass": "march", "drums": "march", "drum_volume": 0.9,
        "staccato": 0.6, "echo": None,
        "sections": [("A", 0.25, False), ("B", 0.5, False), ("A", 0.25, True)],
        "parts": {
            "A": (["C", "G", "Am", "F", "C", "F", "G", "C"], phrase("""
                E5:1 G5:1 C6:2 G5:1 E5:1 G5:2
                D5:1 G5:1 B5:2 A5:1 G5:1 D5:2
                C5:1 E5:1 A5:2 G5:1 E5:1 C5:2
                F5:2 E5:1 F5:1 A5:2 G5:2
                E5:1 G5:1 C6:2 D6:1 C6:1 G5:2
                A5:1 C6:1 F5:2 A5:1 G5:1 F5:2
                D5:1 E5:1 F5:1 G5:1 B5:2 D6:2
                C6:3 G5:1 C5:4
            """), None),
            "B": (["Am", "Em", "F", "C", "Dm", "Am", "E", "E"], phrase("""
                A5:2 B5:1 C6:1 B5:2 A5:2
                G5:2 A5:1 B5:1 E5:4
                F5:2 G5:1 A5:1 C6:2 A5:2
                G5:2 E5:1 C5:1 G5:4
                D5:1 F5:1 A5:1 D6:1 C6:2 A5:2
                C6:1 B5:1 A5:1 E5:1 C5:2 E5:2
                G#5:2 B5:1 G#5:1 E5:2 D5:2
                B4:2 D5:1 E5:1 G#5:2 G5:2
            """), None),
        },
    },
    # two part minuet, D minor, 3/4
    "baroque": {
        "bpm": 132, "bar": 6, "intro": 0, "bass": None, "drums": "minuet", "drum_volume": 0.6,
        "staccato": 0.9, "echo": None,
        "sections": [("A", 0.125, False), ("B", 0.25, False), ("A", 0.125, True), ("B", 0.25, True)],
        "parts": {
            "A": (None, phrase("""
                D5:1 E5:1 F5:1 G5:1 A5:2
                C#5:1 D5:1 E5:1 A4:1 E5:2
                F5:1 G5:1 A5:1 D6:1 A5:2
                G5:1 F5:1 E5:1 D5:1 Bb5:2
                E5:1 F5:1 G5:1 C6:1 G5:2
                A5:1 G5:1 F5:1 E5:1 F5:2
                G5:1 Bb5:1 A5:1 G5:1 F5:1 E5:1
                D5:4 R:2
            """), phrase("""
                D3:2 A2:2 D3:2
                A2:2 C#3:2 A2:2
                D3:2 F3:2 D3:2
                G2:2 Bb2:2 G2:2
                C3:2 E3:2 C3:2
                F2:2 A2:2 F2:2
                G2:2 A2:2 A2:2
                D3:2 A2:2 D2:2
            """)),
            "B": (None, phrase("""
                A5:1 C6:1 A5:1 F5:1 C5:2
                G5:1 E5:1 C5:1 E5:1 G5:2
                F5:1 A5:1 D6:1 C6:1 A5:2
                Bb5:1 A5:1 G5:1 F5:1 D5:2
                G5:1 Bb5:1 D6:1 Bb5:1 G5:2
                A5:1 C#6:1 E6:1 C#6:1 A5:2
                F5:1 E5:1 D5:1 C#5:1 D5:1 E5:1
                A4:4 C#5:1 E5:1
            """), phrase("""
                F2:2 A2:2 C3:2
                C3:2 E3:2 G3:2
                D3:2 F3:2 A3:2
                Bb2:2 D3:2 F3:2
                G2:2 Bb2:2 D3:2
                A2:2 C#3:2 E3:2
                D3:2 A2:2 F2:2
                A2:2 E2:2 A2:2
            """)),
        },
    },
    # slow and moody with echo, E minor
    "nocturne": {
        "bpm": 112, "bar": 8, "intro": 2, "bass": "arp", "drums": "halftime", "drum_volume": 0.55,
        "staccato": 1.0, "echo": (3, 0.35),
        "sections": [("A", 0.5, False), ("B", 0.25, False), ("A", 0.5, True)],
        "parts": {
            "A": (["Em", "C", "G", "D", "Em", "C", "Am", "B"], phrase("""
                B4:4 E5:2 G5:2
                G5:3 F#5:1 E5:4
                D5:4 B4:2 D5:2
                F#5:3 E5:1 D5:4
                B5:4 A5:2 G5:2
                E5:3 G5:1 C6:4
                A5:2 G5:2 E5:2 C5:2
                D#5:6 R:2
            """), None),
            "B": (["C", "D", "Bm", "Em", "Am", "D", "B", "B"], phrase("""
                E5:2 G5:2 C6:4
                D6:3 C6:1 A5:4
                B5:2 F#5:2 D5:4
                E5:3 F#5:1 G5:4
                A5:2 C6:2 E6:4
                D6:2 C6:1 B5:1 A5:4
                B5:2 A5:1 G5:1 F#5:2 D#5:2
                B4:6 R:2
            """), None),
        },
    },
}


# --- arrangement ---------------------------------------------------------------

def render(song):
    rng = random.Random(7)
    eighth = 60 / song["bpm"] / 2
    bar = song["bar"]
    parts = song["parts"]
    bars = song["intro"] + sum(len(parts[name][1]) and bar_count(parts[name][1], bar) for name, _, _ in song["sections"])
    total = bars * bar
    track = array.array("f", [0.0] * (int(total * eighth * RATE) + RATE))

    def length(eighths):
        return int(round(eighths * eighth * RATE))

    def add(start_eighth, samples):
        start = int(round(start_eighth * eighth * RATE))
        for i, s in enumerate(samples):
            track[start + i] += s

    def play_line(at, notes, voice):
        for note, eighths in notes:
            if note != "R":
                voice(at, midi(note), eighths)
            at += eighths
        return at

    drums = DRUM_PATTERNS[song["drums"]]
    dv = song["drum_volume"]

    def add_drums(start):
        for e in drums["kick"]:
            if e < bar:
                add(start + e, kick(rng, dv))
        for e in drums["snare"]:
            if e < bar:
                add(start + e, snare(rng, dv))
        for e in drums["hat"]:
            if e < bar:
                add(start + e, hat(rng, dv))

    def add_chord_bass(start, chord):
        root, third, _ = chord_notes(chord)
        for e, offset in enumerate(BASS_PATTERNS[song["bass"]][:bar]):
            if offset is None:
                continue
            m = root + (third - root + 12 if offset == "3" else offset)
            add(start + e, triangle(freq_midi(m), length(1) - 200, 0.55))

    # intro: drums, plus bass on the first chord of the first section
    first_chords = parts[song["sections"][0][0]][0]
    for b in range(song["intro"]):
        add_drums(b * bar)
        if first_chords:
            add_chord_bass(b * bar, first_chords[0])

    position = song["intro"] * bar
    for name, duty, double in song["sections"]:
        chords, melody, bass_line = parts[name]
        section_bars = bar_count(melody, bar)
        for b in range(section_bars):
            add_drums(position + b * bar)
            if chords:
                add_chord_bass(position + b * bar, chords[b])
        if bass_line:
            end = play_line(position, bass_line, lambda at, m, e: add(at, triangle(freq_midi(m), length(e) - 300, 0.6)))
            assert end - position == section_bars * bar, f"{name} bass length does not match its melody"

        def lead(at, m, e, duty=duty, double=double):
            n = int(length(e) * song["staccato"]) - 300
            add(at, pulse(freq_midi(m), n, duty, 0.32))
            if double:
                add(at, pulse(freq_midi(m - 12), n, 0.125, 0.16))
            if song["echo"]:
                delay, volume = song["echo"]
                if at + delay < total:
                    add(at + delay, pulse(freq_midi(m), min(n, length(total - at - delay) - 1), duty, 0.32 * volume))

        play_line(position, melody, lead)
        position += section_bars * bar

    return track[: length(total)]


def bar_count(melody, bar):
    eighths = sum(e for _, e in melody)
    assert eighths % bar == 0, "melody does not fill whole bars"
    return eighths // bar


def write(name, track):
    peak = max(abs(s) for s in track) or 1.0
    out = OUT_DIR / f"music_{name}.ogg"
    with tempfile.TemporaryDirectory() as tmp:
        wav_path = pathlib.Path(tmp) / "song.wav"
        with wave.open(str(wav_path), "wb") as w:
            w.setnchannels(1)
            w.setsampwidth(2)
            w.setframerate(RATE)
            w.writeframes(array.array("h", (int(s / peak * 0.8 * 32767) for s in track)).tobytes())
        subprocess.run(
            ["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-i", str(wav_path),
             "-c:a", "libvorbis", "-q:a", "6", str(out)],
            check=True,
        )
    print(f"wrote {out.name} ({len(track) / RATE:.1f}s)")


def main():
    names = sys.argv[1:] or list(SONGS)
    for name in names:
        write(name, render(SONGS[name]))


if __name__ == "__main__":
    main()
