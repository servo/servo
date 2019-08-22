#!/usr/bin/python

# Copyright 2016 The Chromium Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.
#
# TODO(50903): Delete the file in LayoutTests/bluetooth after all the tests have
# been migrated to this directory.
"""Test that the set of gen-* files is the same as the generated files."""

import fnmatch
import os
import sys
import generate
import logging

UPDATE_TIP = 'To update the generated tests, run:\n' \
             '$ python third_party/WebKit/LayoutTests/bluetooth/generate.py'


def main():
  logging.basicConfig(level=logging.INFO)
  logging.info(UPDATE_TIP)
  generated_files = set()
  # Tests data in gen-* files is the same as the data generated.
  for generated_test in generate.GetGeneratedTests():
    generated_files.add(generated_test.path)
    try:
      with open(generated_test.path, 'r') as f:
        data = f.read().decode('utf-8')
        if data != generated_test.data:
          logging.error('%s does not match template', generated_test.path)
          return -1
    except IOError, e:
      if e.errno == 2:
        logging.error('Missing generated test:\n%s\nFor template:\n%s',
                     generated_test.path,
                     generated_test.template)
        return -1

  # Tests that there are no obsolete generated files.
  previous_generated_files = set()
  current_path = os.path.dirname(os.path.realpath(__file__))
  for root, _, filenames in os.walk(current_path):
    for filename in fnmatch.filter(filenames, 'gen-*.https.window.js'):
      previous_generated_files.add(os.path.join(root, filename))

  if previous_generated_files != generated_files:
    logging.error('There are extra generated tests. Please remove them.')
    for test_path in previous_generated_files - generated_files:
      logging.error('%s', test_path)
    return -1


if __name__ == '__main__':
  sys.exit(main())
