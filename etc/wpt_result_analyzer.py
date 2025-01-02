#!/usr/bin/env python

# Copyright 2019 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# Usage: python etc/wpt_result_analyzer.py
#
# Analyze the state of WPT tests in Servo by walking all of the
# test directories, counting the number of tests present, and
# counting the number of ini files present in the corresponding
# test result directory. Prints out a list of directories that
# have non-zero failure counts, ordered by overall number of tests
# and percentage of tests that fail.

import os

test_root = os.path.join('tests', 'wpt', 'tests')
meta_root = os.path.join('tests', 'wpt', 'meta')

test_counts = {}
meta_counts = {}

for base_dir, dir_names, files in os.walk(test_root):
    if base_dir == test_root:
        continue

    rel_base = os.path.relpath(base_dir, test_root)
    if not os.path.exists(os.path.join(meta_root, rel_base)):
        continue

    test_files = []
    exts = ['.html', '.htm', '.xht', '.xhtml', '.window.js', '.worker.js', '.any.js']
    for f in files:
        for ext in exts:
            if f.endswith(ext):
                test_files += [f]
    test_counts[rel_base] = len(test_files)

for base_dir, dir_names, files in os.walk(meta_root):
    if base_dir == meta_root:
        continue

    rel_base = os.path.relpath(base_dir, meta_root)
    num_files = len(files)
    if '__dir__.ini' in files:
        num_files -= 1
    meta_counts[rel_base] = num_files

final_counts = []
for (test_dir, test_count) in test_counts.items():
    if not test_count:
        continue
    meta_count = meta_counts.get(test_dir, 0)
    final_counts += [(test_dir, test_count, meta_count)]

print('Test counts')
print('dir: %% failed (num tests / num failures)')
s = sorted(final_counts, key=lambda x: x[2] / x[1])
for (test_dir, test_count, meta_count) in reversed(sorted(s, key=lambda x: x[2])):
    if not meta_count:
        continue
    print('%s: %.2f%% (%d / %d)' % (test_dir, meta_count / test_count * 100, test_count, meta_count))
