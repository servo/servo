#!/usr/bin/env python3

# Copyright 2026 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# usage: python3 untracked-allocations.py [file] [crate-name] [max stacks]
#
# Post-processes the output of a log of untracked allocations from Servo.
# By default, shows the number of untracked allocations attributed to
# each crate from greatest to least.
#
# With additional arguments, this script reports each untracked allocation
# attributed to a backtrace pattern for the given crate, defaulting to the
# top 10 by allocation size. These backtrace patterns are an attempt to
# highlight the most relevant initiator of the allocation, and can be used
# to search for the full backtrace in the unfiltered log file.

import re
import sys

if len(sys.argv) < 2:
    print("usage: untracked-allocations.py file.log [crate-name] [max stacks]")
    sys.exit(0)

with open(sys.argv[1]) as f:
    lines = f.readlines()

crates = {}

# Parts of file paths that precede a crate name.
skippable_patterns = [
    re.compile("/\\.cargo/registry/src/index\\.crates\\.io-[0-9a-f]+/"),
    re.compile(".cargo/git/checkouts/[a-z0-9_\\-]+[0-9a-f]+/[0-9a-f]+/"),
    "/lib/rustlib/src/rust/library/",
    re.compile("/target/\\w+/build/"),
    re.compile("/rustc/[0-9a-f]+/library/"),
    "/rust/deps/",
    "???:0",
    "/components/shared/",
    "/components/",
    "/ports/",
]

# Crates that perform allocations but are ultimately not responsible for them.
# Any backtrace frames that are determined to originate from these crates will
# be skipped when determining the initiator of an allocation.
skippable_crates = [
    "alloc",
    "base",
    "bytes-",
    "core",
    "hashbrown-",
    "indexmap-",
    "servo_arc",
    "slotmap-",
    "smallbitvec-",
    "smallvec-",
    "std",
    "thin-vec-",
]

# Methods that can initiate an allocation but are less interesting than
# their callers.
skippable_methods = [
    "__rust_std_internal_init_fn",
    "ArcRefCell<T>::new",
]

entries = []
current = None
for line in lines:
    line = line.strip()
    # A separator between allocations.
    if line == "---":
        if current:
            if not current["highlight"]:
                current["highlight"] = {"crate": "", "filename": "", "method": ""}
            entries += [current]
        current = {
            "size": 0,
            "highlight": None,
            "frames": [],
        }
        continue

    # The size of a particular allocation.
    if line.isdigit():
        current["size"] = int(line)
        continue

    if not line:
        continue

    current["frames"] += [line]

    # We've already determined a backtrace frame to highlight
    # for this allocation.
    if current["highlight"]:
        continue

    (path, method) = line.split(" ", maxsplit=1)
    original_path = path
    # We're trying to infer a crate name from the file path.
    # If the file path matches one of our known patterns, remove that pattern
    # and everything preceding it.
    for pattern in skippable_patterns:
        if isinstance(pattern, re.Pattern):
            result = re.search(pattern, path)
            if result:
                path = path[result.end() :]
        else:
            index = path.find(pattern)
            if index > -1:
                path = path[index + len(pattern) :]

    # If we've intentionally removed all the data from the file path,
    # this frame is not interesting.
    if not path:
        continue

    crate = path.split("/")[0]
    if not crate:
        print(original_path)
        raise "Error: could not derive crate from path"

    # If the crate name we've determined is not interesting, skip this frame.
    if any([crate.startswith(skippable) for skippable in skippable_crates]):
        continue

    # If the method for this frame is not interesting, skip this frame.
    if any([skippable in method for skippable in skippable_methods]):
        continue

    # The method looks like (<symbol_name>::<hash>), so strip the ()
    method = method[1:-1]
    # Extract the file path from the <path>:<line>
    filename = "/".join(path.split("/")[1:]).split(":")[0]

    # This frame is probably a Servo-originating allocation
    # that we want to use as the attribution.
    current["highlight"] = {"crate": crate, "method": method, "filename": filename}
    if crate not in crates:
        crates[crate] = 0
    crates[crate] += 1

# Print the number of untracked allocations for each crate containing at least
# one untracked allocations.
if len(sys.argv) <= 2:
    print("Overall stats")
    for crate in sorted(crates, key=lambda x: crates[x], reverse=True):
        print(f"{crate}: {crates[crate]}")
    sys.exit(0)

# For a particular crate, print the N largest untracked allocations
# attributed to a particular frame of their allocation backtrace.
relevant = [entry for entry in entries if entry["highlight"]["crate"].startswith(sys.argv[2])]
count = 0
limit = int(sys.argv[3]) if len(sys.argv) > 3 else 10
for entry in sorted(relevant, key=lambda x: x["size"], reverse=True):
    if count == limit:
        break
    print(f"{entry['size']} - {entry['highlight']['method']}")
    count += 1
