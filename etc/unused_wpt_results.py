#!/usr/bin/env python

# Copyright 2019 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# For all directories and ini files under the WPT metadata directory,
# check whether there is a match directory/test file in the vendored WPT
# test collection. If there is not, the test result file is leftover since
# the original test was moved/renamed/deleted and no longer serves any
# purpose.

import os

test_root = os.path.join('tests', 'wpt', 'web-platform-tests')
meta_root = os.path.join('tests', 'wpt', 'metadata')

missing_dirs = []

for base_dir, dir_names, files in os.walk(meta_root):
    # Skip recursing into any directories that were previously found to be missing.
    if base_dir in missing_dirs:
        # Skip processing any subdirectories of known missing directories.
        missing_dirs += map(lambda x: os.path.join(base_dir, x), dir_names)
        continue

    for dir_name in dir_names:
        meta_dir = os.path.join(base_dir, dir_name)

        # Skip any known directories that are meta-metadata.
        if dir_name == '.cache':
            missing_dirs += [meta_dir]
            continue

        # Turn tests/wpt/metadata/foo into tests/wpt/web-platform-tests/foo.
        test_dir = os.path.join(test_root, os.path.relpath(meta_dir, meta_root))
        if not os.path.exists(test_dir):
            missing_dirs += [meta_dir]
            print(meta_dir)

    for fname in files:
        # Skip any known files that are meta-metadata.
        if fname in ['__dir__.ini', 'MANIFEST.json', 'mozilla-sync']:
            continue

        # Turn tests/wpt/metadata/foo/bar.html.ini into tests/wpt/web-platform-tests/foo/bar.html.
        test_dir = os.path.join(test_root, os.path.relpath(base_dir, meta_root))
        test_file = os.path.join(test_dir, fname)
        if not os.path.exists(os.path.splitext(test_file)[0]):
            print(os.path.join(base_dir, fname))
