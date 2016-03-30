# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import unittest
import sys
import tidy
import itertools


class CheckTidiness(unittest.TestCase):
    @classmethod
    def setUpClass(self):
        self.directory = 'python/tidy_self_test/'

    def test_spaces_correctnes(self):
        errors = tidy.collect_errors_for_files([self.directory + 'wrong_space.rs'], [], [tidy.check_by_line])
        self.assertEqual('trailing whitespace', errors.next()[2])
        self.assertEqual('no newline at EOF', errors.next()[2])
        self.assertEqual('tab on line', errors.next()[2])
        self.assertEqual('CR on line', errors.next()[2])

    def test_long_line(self):
        errors = tidy.collect_errors_for_files([self.directory + 'long_line.rs'], [], [tidy.check_by_line])
        self.assertEqual('Line is longer than 120 characters', errors.next()[2])

    def test_whatwg_link(self):
        errors = tidy.collect_errors_for_files([self.directory + 'whatwg_link.rs'], [], [tidy.check_by_line])
        self.assertEqual('link to WHATWG may break in the future, use this format instead: https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata', errors.next()[2])
        self.assertEqual('links to WHATWG single-page url, change to multi page: https://html.spec.whatwg.org/multipage/#typographic-conventions', errors.next()[2])

    def test_licence(self):
        errors = tidy.collect_errors_for_files([self.directory + 'incorrect_license.rs'], [], [tidy.check_license])
        self.assertEqual('incorrect license', errors.next()[2])

    def test_rust(self):
        errors = tidy.collect_errors_for_files([self.directory + 'rust_tidy.rs'], [], [tidy.check_rust])
        self.assertEqual('use statement spans multiple lines', errors.next()[2])
        self.assertEqual('missing space before }', errors.next()[2])
        self.assertTrue('use statement is not in alphabetical order' in errors.next()[2])
        self.assertEqual('missing space before ->', errors.next()[2])
        self.assertEqual('missing space after ->', errors.next()[2])
        self.assertEqual('missing space after :', errors.next()[2])
        self.assertEqual('missing space before {', errors.next()[2])
        self.assertEqual('missing space before =', errors.next()[2])
        self.assertEqual('missing space after =', errors.next()[2])
        self.assertEqual('missing space before -', errors.next()[2])
        self.assertEqual('missing space before *', errors.next()[2])
        self.assertEqual('missing space after =>', errors.next()[2])
        self.assertEqual('extra space before :', errors.next()[2])
        self.assertEqual('extra space before :', errors.next()[2])
        self.assertEqual('use &[T] instead of &Vec<T>', errors.next()[2])
        self.assertEqual('use &str instead of &String', errors.next()[2])
        # TODO missing tests for:
        # extern crate declaration
        # encountered whitespace following a use statement
        # mod declaration spans multiple lines

    def test_webidl(self):
        errors = tidy.collect_errors_for_files([self.directory + 'spec.webidl'], [tidy.check_webidl_spec], [])
        self.assertEqual('No specification link found.', errors.next()[2])


def print_usage():
    print("USAGE: python {0}".format(sys.argv[0]))


def doTests():
    suite = unittest.TestLoader().loadTestsFromTestCase(CheckTidiness)
    unittest.TextTestRunner(verbosity=2).run(suite)
