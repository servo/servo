# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import sys
import unittest
sys.path.insert(0, '~/servo/tests/wpt/harness/wptrunner/executors/')
import webdriver
import subprocess


class TestStringMethods(unittest.TestCase):
    def test_get(self):
        proc = subprocess.Popen(["./mach run --webdriver 8000 tests/html/about-mozilla.html"], shell=True)
        session = webdriver.Session('127.0.0.1', 8000)
        session.start()
        session.url = "http://google.com/about"
        a = self.assertEqual(session.url, "https://www.google.com/about/")
        if a:
            session.end()
            proc.kill()

    def test_get1(self):
        proc = subprocess.Popen(["./mach run --webdriver 6000 tests/html/about-mozilla.html"], shell=True)
        session = webdriver.Session('127.0.0.1', 6000)
        session.start()
        session.url = "http://google.com"
        a = self.assertEqual(session.url, "https://www.google.com/")
        if a:
            session.end()
            proc.kill()

if __name__ == '__main__':
    unittest.main()
