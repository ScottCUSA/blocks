#!/usr/bin/env python3
"""Generate the game's 8-bit style sound effects into resources/ as 16-bit mono WAV files."""

import math
import pathlib
import random
import struct
import wave

RATE = 44100
PITCH = 0.5  # scales every frequency, 0.5 is an octave down
OUT_DIR = pathlib.Path(__file__).resolve().parent.parent / "resources"
NOTE = {
    "C4": 261.63, "G4": 392.00, "C5": 523.25, "E5": 659.25, "G5": 783.99,
    "B5": 987.77, "C6": 1046.50, "E6": 1318.51, "G6": 1567.98,
}


def steps(value, levels):
    """quantize 0..1 to a number of levels, like the NES 4-bit volume"""
    return round(value * (levels - 1)) / (levels - 1)


def tone(freq, dur, end_freq=None, shape="pulse", duty=0.5, decay=6.0, volume=1.0):
    end_freq = (end_freq or freq) * PITCH
    freq *= PITCH
    n = int(RATE * dur)
    out, phase = [], 0.0
    for i in range(n):
        t = i / n
        phase = (phase + (freq + (end_freq - freq) * t) / RATE) % 1.0
        if shape == "pulse":
            s = 1.0 if phase < duty else -1.0
        else:  # stepped triangle, 16 levels
            s = steps(1 - abs(phase * 2 - 1), 16) * 2 - 1
        out.append(s * steps(math.exp(-decay * t), 16) * volume)
    return out


def noise(dur, hold=8, decay=10.0, volume=1.0, seed=1):
    """random levels held for a number of samples, like the NES noise channel"""
    rng = random.Random(seed)
    n = int(RATE * dur)
    out, level = [], 0.0
    for i in range(n):
        if i % hold == 0:
            level = rng.choice((-1.0, 1.0))
        out.append(level * steps(math.exp(-decay * i / n), 16) * volume)
    return out


def mix(*parts):
    """parts are (offset_seconds, samples)"""
    length = max(int(off * RATE) + len(s) for off, s in parts)
    out = [0.0] * length
    for off, samples in parts:
        start = int(off * RATE)
        for i, s in enumerate(samples):
            out[start + i] += s
    return out


def note(name, dur, duty=0.25, volume=1.0, semitones=0):
    """pulse melody with a triangle bass an octave below"""
    f = NOTE[name] * 2 ** (semitones / 12)
    return mix(
        (0, tone(f, dur, duty=duty, decay=4, volume=volume)),
        (0, tone(f / 2, dur, shape="triangle", decay=3, volume=volume * 0.7)),
    )


def blip(freq, dur, end_freq=None, duty=0.25, decay=6.0):
    """pulse with a triangle bass an octave below"""
    return mix(
        (0, tone(freq, dur, end_freq, duty=duty, decay=decay)),
        (0, tone(freq / 2, dur, end_freq and end_freq / 2, shape="triangle", decay=decay, volume=0.7)),
    )


def line_clear(semitones):
    """rising fanfare, raised for each extra line cleared"""
    return mix(*[
        (i * 0.09, note(n, 0.12 if i < 3 else 0.32, duty=0.5, semitones=semitones))
        for i, n in enumerate(["G4", "C5", "E5", "G5"])
    ])


def level_up():
    """fast thin scale run with a rising bass, then a fading sparkle trill"""
    scale = [523.25, 587.33, 659.25, 698.46, 783.99, 880.00, 987.77, 1046.50]
    step = 0.035
    run = [(i * step, tone(f, step, duty=0.125, decay=2)) for i, f in enumerate(scale)]
    sweep = (0, tone(130.81, step * len(scale), end_freq=261.63, shape="triangle", decay=1, volume=0.8))
    start = step * len(scale)
    trill = [
        (start + i * 0.03, tone(1318.51 if i % 2 == 0 else 1567.98, 0.03, duty=0.25, decay=0, volume=0.9 ** i))
        for i in range(10)
    ]
    hold = (start, tone(261.63, 0.3, shape="triangle", decay=4, volume=0.7))
    return mix(*run, sweep, *trill, hold)


def high_score():
    """brass style fanfare, repeated pickups into a held note, then a climb to a held chord"""
    beat = 0.08
    # (beat, length in beats, melody, harmony, bass)
    score = [
        (0, 0.75, "G4", None, "G4"), (1, 0.75, "G4", None, None), (2, 0.75, "G4", None, None),
        (3, 3, "C5", "G4", "C4"),
        (6, 0.75, "E5", "C5", None), (7, 0.75, "G5", "E5", None),
        (8, 8, "C6", "G5", "C4"),
    ]
    parts = []
    for at, length, melody, harmony, bass in score:
        t, dur = at * beat, length * beat
        decay = 1.5 if length > 1 else 2
        parts.append((t, tone(NOTE[melody], dur, duty=0.5, decay=decay)))
        if harmony:
            parts.append((t, tone(NOTE[harmony], dur, duty=0.25, decay=decay, volume=0.6)))
        if bass:
            parts.append((t, tone(NOTE[bass] / 2, dur if length > 1 else 3 * beat, shape="triangle", decay=2, volume=0.8)))
    return mix(*parts)


def write(name, samples, peak=0.6):
    top = max(abs(s) for s in samples) or 1.0
    frames = bytearray()
    for s in samples:
        frames += struct.pack("<h", int(s / top * peak * 32767))
    with wave.open(str(OUT_DIR / f"{name}.wav"), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(RATE)
        w.writeframes(bytes(frames))
    print(f"wrote {name}.wav ({len(samples) / RATE:.2f}s)")


SOUNDS = {
    "move": (blip(1200, 0.03, duty=0.125, decay=10), 0.25),
    "rotate": (blip(600, 0.06, end_freq=1200, decay=5), 0.3),
    "lock": (mix((0, tone(260, 0.12, end_freq=110, shape="triangle", decay=5)), (0, noise(0.08, hold=30, decay=8, volume=0.5, seed=3))), 0.5),
    "hard_drop": (mix((0, tone(180, 0.18, end_freq=50, shape="triangle", decay=4)), (0, noise(0.12, hold=40, decay=7, volume=0.6, seed=2))), 0.6),
    "hold": (mix((0, blip(784, 0.05, duty=0.5, decay=4)), (0.05, blip(523, 0.08, duty=0.5, decay=5))), 0.35),
    "line_clear": (line_clear(0), 0.3),
    "line_clear_2": (line_clear(2), 0.3),
    "line_clear_3": (line_clear(4), 0.3),
    "quad": (
        mix(
            (0, note("C5", 0.07)), (0.06, note("E5", 0.07)), (0.12, note("G5", 0.07)),
            (0.18, note("C6", 0.07)), (0.24, note("E6", 0.07)), (0.30, note("G6", 0.38)),
            (0.30, tone(NOTE["C4"], 0.38, shape="triangle", decay=3, volume=0.8)),
        ),
        0.55,
    ),
    "level_up": (level_up(), 0.2),
    "high_score": (high_score(), 0.7),  # played at jingle volume
    "menu_move": (blip(988, 0.04, decay=8), 0.25),
    "menu_select": (mix((0, blip(659, 0.05, decay=3)), (0.05, blip(1319, 0.11, decay=4))), 0.35),
}

if __name__ == "__main__":
    for name, (samples, peak) in SOUNDS.items():
        write(name, samples, peak)
