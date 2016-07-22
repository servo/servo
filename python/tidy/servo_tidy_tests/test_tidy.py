# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import os
import unittest
from servo_tidy import tidy

base_path = 'servo_tidy_tests/' if os.path.exists('servo_tidy_tests/') else 'python/tidy/servo_tidy_tests/'


def iterFile(name):
    return iter([os.path.join(base_path, name)])


class CheckTidiness(unittest.TestCase):
    def assertNoMoreErrors(self, errors):
        with self.assertRaises(StopIteration):
            errors.next()

    def test_spaces_correctnes(self):
        errors = tidy.collect_errors_for_files(iterFile('wrong_space.rs'), [], [tidy.check_by_line], print_text=False)
        self.assertEqual('trailing whitespace', errors.next()[2])
        self.assertEqual('no newline at EOF', errors.next()[2])
        self.assertEqual('tab on line', errors.next()[2])
        self.assertEqual('CR on line', errors.next()[2])
        self.assertEqual('no newline at EOF', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_long_line(self):
        errors = tidy.collect_errors_for_files(iterFile('long_line.rs'), [], [tidy.check_by_line], print_text=False)
        self.assertEqual('Line is longer than 120 characters', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_whatwg_link(self):
        errors = tidy.collect_errors_for_files(iterFile('whatwg_link.rs'), [], [tidy.check_by_line], print_text=False)
        self.assertTrue('link to WHATWG may break in the future, use this format instead:' in errors.next()[2])
        self.assertTrue('links to WHATWG single-page url, change to multi page:' in errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_licence(self):
        errors = tidy.collect_errors_for_files(iterFile('incorrect_license.rs'), [], [tidy.check_license], print_text=False)
        self.assertEqual('incorrect license', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_shell(self):
        errors = tidy.collect_errors_for_files(iterFile('shell_tidy.sh'), [], [tidy.check_shell], print_text=False)
        self.assertEqual('script does not have shebang "#!/usr/bin/env bash"', errors.next()[2])
        self.assertEqual('script is missing options "set -o errexit", "set -o pipefail"', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_rust(self):
        errors = tidy.collect_errors_for_files(iterFile('rust_tidy.rs'), [], [tidy.check_rust], print_text=False)
        self.assertEqual('use statement spans multiple lines', errors.next()[2])
        self.assertEqual('missing space before }', errors.next()[2])
        self.assertTrue('use statement is not in alphabetical order' in errors.next()[2])
        self.assertEqual('use statement contains braces for single import', errors.next()[2])
        self.assertEqual('encountered whitespace following a use statement', errors.next()[2])
        self.assertTrue('mod declaration is not in alphabetical order' in errors.next()[2])
        self.assertEqual('mod declaration spans multiple lines', errors.next()[2])
        self.assertTrue('extern crate declaration is not in alphabetical order' in errors.next()[2])
        self.assertEqual('found an empty line following a {', errors.next()[2])
        self.assertEqual('missing space before ->', errors.next()[2])
        self.assertEqual('missing space after ->', errors.next()[2])
        self.assertEqual('missing space after :', errors.next()[2])
        self.assertEqual('missing space before {', errors.next()[2])
        self.assertEqual('missing space before =', errors.next()[2])
        self.assertEqual('missing space after =', errors.next()[2])
        self.assertEqual('missing space before -', errors.next()[2])
        self.assertEqual('missing space before *', errors.next()[2])
        self.assertEqual('missing space after =>', errors.next()[2])
        self.assertEqual('missing space after :', errors.next()[2])
        self.assertEqual('missing space after :', errors.next()[2])
        self.assertEqual('extra space before :', errors.next()[2])
        self.assertEqual('extra space before :', errors.next()[2])
        self.assertEqual('use &[T] instead of &Vec<T>', errors.next()[2])
        self.assertEqual('use &str instead of &String', errors.next()[2])
        self.assertEqual('use &T instead of &Root<T>', errors.next()[2])
        self.assertEqual('operators should go at the end of the first line', errors.next()[2])
        self.assertEqual('else braces should be on the same line', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_spec_link(self):
        tidy.SPEC_BASE_PATH = base_path
        errors = tidy.collect_errors_for_files(iterFile('speclink.rs'), [], [tidy.check_spec], print_text=False)
        self.assertEqual('method declared in webidl is missing a comment with a specification link', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_webidl(self):
        errors = tidy.collect_errors_for_files(iterFile('spec.webidl'), [tidy.check_webidl_spec], [], print_text=False)
        self.assertEqual('No specification link found.', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_toml(self):
        errors = tidy.collect_errors_for_files(iterFile('test.toml'), [tidy.check_toml], [], print_text=False)
        self.assertEqual('found asterisk instead of minimum version number', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_modeline(self):
        errors = tidy.collect_errors_for_files(iterFile('modeline.txt'), [], [tidy.check_modeline], print_text=False)
        self.assertEqual('vi modeline present', errors.next()[2])
        self.assertEqual('vi modeline present', errors.next()[2])
        self.assertEqual('vi modeline present', errors.next()[2])
        self.assertEqual('emacs file variables present', errors.next()[2])
        self.assertEqual('emacs file variables present', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_malformed_json(self):
        errors = tidy.collect_errors_for_files(iterFile('malformed_json.json'), [tidy.check_json], [], print_text=False)
        self.assertEqual('Invalid control character at: line 3 column 40 (char 61)', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_json_with_duplicate_key(self):
        errors = tidy.collect_errors_for_files(iterFile('duplicate_key.json'), [tidy.check_json], [], print_text=False)
        self.assertEqual('Duplicated Key (the_duplicated_key)', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_lock(self):
        errors = tidy.collect_errors_for_files(iterFile('duplicated_package.lock'), [tidy.check_lock], [], print_text=False)
        msg = """duplicate versions for package "test"
\t\033[93mfound dependency on version 0.4.9\033[0m
\t\033[91mbut highest version is 0.5.1\033[0m
\t\033[93mtry upgrading with\033[0m \033[96m./mach cargo-update -p test:0.4.9\033[0m
\tThe following packages depend on version 0.4.9:
\t\ttest2"""
        self.assertEqual(msg, errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_file_list(self):
        base_path='./python/tidy/servo_tidy_tests/test_ignored'
        file_list = tidy.get_file_list(base_path, only_changed_files=False,
                                       exclude_dirs=[])
        lst = list(file_list)
        self.assertEqual([os.path.join(base_path, 'whee', 'test.rs')], lst)
        file_list = tidy.get_file_list(base_path, only_changed_files=False,
                                       exclude_dirs=[os.path.join(base_path,'whee')])
        lst = list(file_list)
        self.assertEqual([], lst)

def do_tests():
    suite = unittest.TestLoader().loadTestsFromTestCase(CheckTidiness)
    unittest.TextTestRunner(verbosity=2).run(suite)
