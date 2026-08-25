#!/usr/bin/env python

import getpass
import json
import pickle
import subprocess
import sys
import time
import zlib


CORPUS_FILE = "corpus.pkl"

REPEAT = 10

WB, ML = 12, 5  # defaults used as a reference


def _corpus():
    OAUTH_TOKEN = getpass.getpass("OAuth Token? ")
    COMMIT_API = (
        f'curl -H "Authorization: token {OAUTH_TOKEN}" '
        f"https://api.github.com/repos/python-websockets/websockets/git/commits/:sha"
    )

    commits = []

    head = subprocess.check_output("git rev-parse HEAD", shell=True).decode().strip()
    todo = [head]
    seen = set()

    while todo:
        sha = todo.pop(0)
        commit = subprocess.check_output(COMMIT_API.replace(":sha", sha), shell=True)
        commits.append(commit)
        seen.add(sha)
        for parent in json.loads(commit)["parents"]:
            sha = parent["sha"]
            if sha not in seen and sha not in todo:
                todo.append(sha)
        time.sleep(1)  # rate throttling

    return commits


def corpus():
    data = _corpus()
    with open(CORPUS_FILE, "wb") as handle:
        pickle.dump(data, handle)


def _run(data):
    size = {}
    duration = {}

    for wbits in range(9, 16):
        size[wbits] = {}
        duration[wbits] = {}

        for memLevel in range(1, 10):
            encoder = zlib.compressobj(wbits=-wbits, memLevel=memLevel)
            encoded = []

            t0 = time.perf_counter()

            for _ in range(REPEAT):
                for item in data:
                    if isinstance(item, str):
                        item = item.encode("utf-8")
                    # Taken from PerMessageDeflate.encode
                    item = encoder.compress(item) + encoder.flush(zlib.Z_SYNC_FLUSH)
                    if item.endswith(b"\x00\x00\xff\xff"):
                        item = item[:-4]
                    encoded.append(item)

            t1 = time.perf_counter()

            size[wbits][memLevel] = sum(len(item) for item in encoded)
            duration[wbits][memLevel] = (t1 - t0) / REPEAT

    raw_size = sum(len(item) for item in data)

    print("=" * 79)
    print("Compression ratio")
    print("=" * 79)
    print("\t".join(["wb \\ ml"] + [str(memLevel) for memLevel in range(1, 10)]))
    for wbits in range(9, 16):
        print(
            "\t".join(
                [str(wbits)]
                + [
                    f"{100 * (1 - size[wbits][memLevel] / raw_size):.1f}%"
                    for memLevel in range(1, 10)
                ]
            )
        )
    print("=" * 79)
    print()

    print("=" * 79)
    print("CPU time")
    print("=" * 79)
    print("\t".join(["wb \\ ml"] + [str(memLevel) for memLevel in range(1, 10)]))
    for wbits in range(9, 16):
        print(
            "\t".join(
                [str(wbits)]
                + [
                    f"{1000 * duration[wbits][memLevel]:.1f}ms"
                    for memLevel in range(1, 10)
                ]
            )
        )
    print("=" * 79)
    print()

    print("=" * 79)
    print(f"Size vs. {WB} \\ {ML}")
    print("=" * 79)
    print("\t".join(["wb \\ ml"] + [str(memLevel) for memLevel in range(1, 10)]))
    for wbits in range(9, 16):
        print(
            "\t".join(
                [str(wbits)]
                + [
                    f"{100 * (size[wbits][memLevel] / size[WB][ML] - 1):.1f}%"
                    for memLevel in range(1, 10)
                ]
            )
        )
    print("=" * 79)
    print()

    print("=" * 79)
    print(f"Time vs. {WB} \\ {ML}")
    print("=" * 79)
    print("\t".join(["wb \\ ml"] + [str(memLevel) for memLevel in range(1, 10)]))
    for wbits in range(9, 16):
        print(
            "\t".join(
                [str(wbits)]
                + [
                    f"{100 * (duration[wbits][memLevel] / duration[WB][ML] - 1):.1f}%"
                    for memLevel in range(1, 10)
                ]
            )
        )
    print("=" * 79)
    print()


def run():
    with open(CORPUS_FILE, "rb") as handle:
        data = pickle.load(handle)
    _run(data)


try:
    run = globals()[sys.argv[1]]
except (KeyError, IndexError):
    print(f"Usage: {sys.argv[0]} [corpus|run]")
else:
    run()
