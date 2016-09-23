#!/usr/bin/env python

import difflib
import json
import os
import subprocess
import sys


def call(*args):
    return subprocess.check_output(args)


def get_manifest(rev):
    call("git", "checkout", rev)
    call("./manifest", "-r")
    with open("MANIFEST.json", "r") as fp:
        return fp.readlines()


def main():
    head = os.environ['TRAVIS_COMMIT']
    call("git", "fetch", "origin", "master:master")
    merge_base = call("git", "merge-base", "master", head).strip()
    before = get_manifest(merge_base)
    after = get_manifest(head)
    diff = difflib.unified_diff(before, after,
                                fromfile='before.json', tofile='after.json')
    for line in diff:
        sys.stdout.write(line)


if __name__ == "__main__":
    main()
