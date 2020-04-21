#!/usr/bin/env python3

# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import json
import os
import subprocess
import sys


def main(crate=None):
    os.chdir(os.path.join(os.path.dirname(__file__), ".."))
    meta = json.loads(subprocess.check_output(["cargo", "metadata", "--format-version", "1"]))
    graph = {}
    for package in meta["packages"]:
        if package["source"] is None:  # Lives in this repo
            for dependency in package["dependencies"]:
                if dependency["source"] is None:  # Also lives in this repo
                    graph.setdefault(package["name"], []).append(dependency["name"])

    if crate:
        filtered = {}
        seen = set()

        def traverse(name):
            if name not in seen:
                seen.add(name)
                for dependency in graph.get(name, []):
                    filtered.setdefault(name, []).append(dependency)
                    traverse(dependency)
        traverse(crate)
    else:
        filtered = graph
    print("// This is in Graphviz DOT format.")
    print("// Use the 'dot' or 'xdot' tool to visualize.")
    print('digraph "local crates" {')
    for package, dependencies in filtered.items():
        for dependency in dependencies:
            print('  "%s" -> "%s";' % (package, dependency))
    print("}")


if __name__ == "__main__":
    sys.exit(main(*sys.argv[1:]))
