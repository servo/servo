#!/usr/bin/env python

# Copyright 2023 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import unittest
from try_parser import Config


class TestParser(unittest.TestCase):
    def test_string(self):
        self.assertEqual(Config("linux").toJSON(),
                         '{"fail_fast": false, "matrix": [{"os": "linux",\
 "name": "Linux", "wpt_layout": "none", "profile": "release", "unit_tests": true, "wpt_tests_to_run": ""}]}')

    def test_tuple1(self):
        conf = Config("linux(profile='debug')")
        self.assertEqual(conf.matrix[0].profile, "debug")

    def test_tuple2(self):
        conf = Config("linux(profile=debug, unit-tests=false)")
        self.assertEqual(conf.matrix[0].profile, 'debug')
        self.assertEqual(conf.matrix[0].unit_tests, False)

    def test_special(self):
        conf = Config("fail-fast try")
        self.assertEqual(conf.fail_fast, True)
        self.assertEqual(len(conf.matrix), 3)


if __name__ == "__main__":
    unittest.main()
