# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import unittest
import tidy


def iterFile(name):
    return iter(['python/tidy_self_test/' + name])


class CheckTidiness(unittest.TestCase):
    def test_spaces_correctnes(self):
        errors = tidy.collect_errors_for_files(iterFile('wrong_space.rs'), [], [tidy.check_by_line])
        self.assertEqual('trailing whitespace', errors.next()[2])
        self.assertEqual('no newline at EOF', errors.next()[2])
        self.assertEqual('tab on line', errors.next()[2])
        self.assertEqual('CR on line', errors.next()[2])

    def test_long_line(self):
        errors = tidy.collect_errors_for_files(iterFile('long_line.rs'), [], [tidy.check_by_line])
        self.assertEqual('Line is longer than 120 characters', errors.next()[2])

    def test_whatwg_link(self):
        errors = tidy.collect_errors_for_files(iterFile('whatwg_link.rs'), [], [tidy.check_by_line])
        self.assertTrue('link to WHATWG may break in the future, use this format instead:' in errors.next()[2])
        self.assertTrue('links to WHATWG single-page url, change to multi page:' in errors.next()[2])

    def test_licence(self):
        errors = tidy.collect_errors_for_files(iterFile('incorrect_license.rs'), [], [tidy.check_license])
        self.assertEqual('incorrect license', errors.next()[2])

    def test_rust(self):
        errors = tidy.collect_errors_for_files(iterFile('rust_tidy.rs'), [], [tidy.check_rust])
        self.assertEqual('use statement spans multiple lines', errors.next()[2])
        self.assertEqual('missing space before }', errors.next()[2])
        self.assertTrue('use statement is not in alphabetical order' in errors.next()[2])
        self.assertEqual('encountered whitespace following a use statement', errors.next()[2])
        self.assertTrue('mod declaration is not in alphabetical order' in errors.next()[2])
        self.assertEqual('mod declaration spans multiple lines', errors.next()[2])
        self.assertTrue('extern crate declaration is not in alphabetical order' in errors.next()[2])
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

    def test_webidl(self):
        errors = tidy.collect_errors_for_files(iterFile('spec.webidl'), [tidy.check_webidl_spec], [])
        self.assertEqual('No specification link found.', errors.next()[2])

    def test_toml(self):
        errors = tidy.collect_errors_for_files(iterFile('test.toml'), [tidy.check_toml], [])
        self.assertEqual('found asterisk instead of minimum version number', errors.next()[2])


def do_tests():
    suite = unittest.TestLoader().loadTestsFromTestCase(CheckTidiness)
    unittest.TextTestRunner(verbosity=2).run(suite)
