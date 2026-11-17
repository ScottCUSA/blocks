#!/usr/bin/env python3
"""Summarize or compare frame stats recordings from the debug viewer (C to record).

    tools/compare_frames.py frames-A.csv              per scene summary
    tools/compare_frames.py frames-A.csv frames-B.csv B against A, per scene in both

Rows are grouped by scene (animation, background, block, board, level), so runs line up
even when the scenes were visited in a different order or for different lengths.
"""

import argparse
import csv
import statistics
import sys

SCENE = ("animation", "background", "block", "board", "level")
# header fields that should match for a fair comparison
COMPARABLE = ("profile", "os", "drawable", "fps_cap", "format")


def load(path):
    info, lines = {}, []
    with open(path, newline="") as f:
        for line in f:
            if line.startswith("# blocks frame stats"):
                info["format"] = line.split()[-1]
            elif line.startswith("# "):
                for part in line[2:].split():
                    key, _, value = part.partition("=")
                    if value:
                        info[key] = value
            else:
                lines.append(line)
    return info, list(csv.DictReader(lines))


def scenes(rows, warmup):
    """samples per scene, skipping the first frames after each scene change"""
    grouped, last, run = {}, None, 0
    for row in rows:
        scene = tuple(row[k] for k in SCENE)
        run = run + 1 if scene == last else 0
        last = scene
        if run < warmup:
            continue
        sample = grouped.setdefault(scene, {"frame": [], "cpu": [], "draws": []})
        sample["frame"].append(float(row["frame_ms"]))
        sample["cpu"].append(float(row["scene_ms"]))
        sample["draws"].append(float(row["draws"]))
    return grouped


def p95(values):
    values = sorted(values)
    return values[min(len(values) - 1, int(len(values) * 0.95))]


def name(scene):
    animation, background, block, board, level = scene
    return f"{animation} / {background} / {block} / {board} / L{level}"


def summary(info, grouped):
    print(f"commit {info.get('commit', '?')}  {info.get('profile', '?')}  "
          f"{info.get('drawable', '?')}  cap {info.get('fps_cap', '?')}")
    print(f"{'scene':<52} {'n':>5} {'draws':>6} {'scene ms':>8} {'p95':>7} {'frame ms':>9} {'p95':>7}")
    for scene in sorted(grouped):
        s = grouped[scene]
        print(f"{name(scene):<52} {len(s['cpu']):>5} {statistics.median(s['draws']):>6.0f} "
              f"{statistics.median(s['cpu']):>8.2f} {p95(s['cpu']):>7.2f} "
              f"{statistics.median(s['frame']):>9.2f} {p95(s['frame']):>7.2f}")


def change(a, b):
    return f"{(b - a) / a * 100:+.0f}%" if a else "n/a"


def compare(info_a, a, info_b, b):
    for key in COMPARABLE:
        if info_a.get(key) != info_b.get(key):
            print(f"warning: {key} differs: {info_a.get(key)} vs {info_b.get(key)}", file=sys.stderr)
    if "debug" in (info_a.get("profile"), info_b.get("profile")):
        print("warning: debug build timings are not representative", file=sys.stderr)
    print(f"A {info_a.get('commit', '?')}  ->  B {info_b.get('commit', '?')}  (medians)")
    print(f"{'scene':<52} {'draws A':>7} {'B':>6} {'cpu A':>6} {'B':>6} {'':>5} "
          f"{'frm A':>6} {'B':>6} {'':>5}")
    shared = sorted(set(a) & set(b))
    for scene in shared:
        sa, sb = a[scene], b[scene]
        med = lambda s, k: statistics.median(s[k])
        print(f"{name(scene):<52} {med(sa, 'draws'):>7.0f} {med(sb, 'draws'):>6.0f} "
              f"{med(sa, 'cpu'):>6.2f} {med(sb, 'cpu'):>6.2f} {change(med(sa, 'cpu'), med(sb, 'cpu')):>5} "
              f"{med(sa, 'frame'):>6.2f} {med(sb, 'frame'):>6.2f} "
              f"{change(med(sa, 'frame'), med(sb, 'frame')):>5}")
    for label, only in (("A", set(a) - set(b)), ("B", set(b) - set(a))):
        for scene in sorted(only):
            print(f"only in {label}: {name(scene)}")
    if not shared:
        print("no scenes in common")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("a")
    parser.add_argument("b", nargs="?")
    parser.add_argument("--warmup", type=int, default=30, help="frames skipped after each scene change")
    args = parser.parse_args()
    info_a, rows_a = load(args.a)
    if args.b is None:
        summary(info_a, scenes(rows_a, args.warmup))
        return
    info_b, rows_b = load(args.b)
    compare(info_a, scenes(rows_a, args.warmup), info_b, scenes(rows_b, args.warmup))


if __name__ == "__main__":
    main()
