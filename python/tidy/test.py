# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import logging
import os
import unittest

from . import tidy


BASE_PATH = 'python/tidy/tests/'

def test_file_path(name):
    return os.path.join(BASE_PATH, name)


def iterFile(name):
    return iter([test_file_path(name)])


class CheckTidiness(unittest.TestCase):
    def assertNoMoreErrors(self, errors):
        with self.assertRaises(StopIteration):
            next(errors)

    def test_tidy_config(self):
        errors = tidy.check_config_file(os.path.join(BASE_PATH, 'servo-tidy.toml'), print_text=False)
        self.assertEqual("invalid config key 'key-outside'", next(errors)[2])
        self.assertEqual("invalid config key 'wrong-key'", next(errors)[2])
        self.assertEqual('invalid config table [wrong]', next(errors)[2])
        self.assertEqual("ignored file './fake/file.html' doesn't exist", next(errors)[2])
        self.assertEqual("ignored directory './fake/dir' doesn't exist", next(errors)[2])
        self.assertNoMoreErrors(errors)

    def test_directory_checks(self):
        dirs = {
            os.path.join(BASE_PATH, "dir_check/webidl_plus"): ['webidl', 'test'],
            os.path.join(BASE_PATH, "dir_check/only_webidl"): ['webidl']
        }
        errors = tidy.check_directory_files(dirs, print_text=False)
        error_dir = os.path.join(BASE_PATH, "dir_check/webidl_plus")
        self.assertEqual("Unexpected extension found for test.rs. We only expect files with webidl, test extensions in {0}".format(error_dir), next(errors)[2])
        self.assertEqual("Unexpected extension found for test2.rs. We only expect files with webidl, test extensions in {0}".format(error_dir), next(errors)[2])
        self.assertNoMoreErrors(errors)

    def test_spaces_correctnes(self):
        errors = tidy.collect_errors_for_files(iterFile('wrong_space.rs'), [], [tidy.check_by_line], print_text=False)
        self.assertEqual('trailing whitespace', next(errors)[2])
        self.assertEqual('no newline at EOF', next(errors)[2])
        self.assertEqual('tab on line', next(errors)[2])
        self.assertEqual('CR on line', next(errors)[2])
        self.assertEqual('no newline at EOF', next(errors)[2])
        self.assertNoMoreErrors(errors)

    def test_empty_file(self):
        errors = tidy.collect_errors_for_files(iterFile('empty_file.rs'), [], [tidy.check_by_line], print_text=False)
        self.assertEqual('file is empty', next(errors)[2])
        self.assertNoMoreErrors(errors)

    def test_long_line(self):
        errors = tidy.collect_errors_for_files(iterFile('long_line.rs'), [], [tidy.check_by_line], print_text=False)
        self.assertEqual('Line is longer than 120 characters', next(errors)[2])
        self.assertNoMoreErrors(errors)

    def test_whatwg_link(self):
        errors = tidy.collect_errors_for_files(iterFile('whatwg_link.rs'), [], [tidy.check_by_line], print_text=False)
        self.assertEqual('link to WHATWG may break in the future, use this format instead: https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata', next(errors)[2])
        self.assertEqual('links to WHATWG single-page url, change to multi page: https://html.spec.whatwg.org/multipage/#typographic-conventions', next(errors)[2])
        self.assertNoMoreErrors(errors)

    def test_license(self):
        errors = tidy.collect_errors_for_files(iterFile('incorrect_license.rs'), [], [tidy.check_license], print_text=False)
        self.assertEqual('incorrect license', next(errors)[2])
        self.assertNoMoreErrors(errors)

    def test_shebang_license(self):
        errors = tidy.collect_errors_for_files(iterFile('shebang_license.py'), [], [tidy.check_license], print_text=False)
        self.assertEqual('missing blank line after shebang', next(errors)[2])
        self.assertNoMoreErrors(errors)

    def test_shell(self):
        errors = tidy.collect_errors_for_files(iterFile('shell_tidy.sh'), [], [tidy.check_shell], print_text=False)
        self.assertEqual('script does not have shebang "#!/usr/bin/env bash"', next(errors)[2])
        self.assertEqual('script is missing options "set -o errexit", "set -o pipefail"', next(errors)[2])
        self.assertEqual('script should not use backticks for command substitution', next(errors)[2])
        self.assertEqual('variable substitutions should use the full \"${VAR}\" form', next(errors)[2])
        self.assertEqual('script should use `[[` instead of `[` for conditional testing', next(errors)[2])
        self.assertEqual('script should use `[[` instead of `[` for conditional testing', next(errors)[2])
        self.assertNoMoreErrors(errors)

    def test_apache2_incomplete(self):
        errors = tidy.collect_errors_for_files(iterFile('apache2_license.rs'), [], [tidy.check_license], print_text=False)
        self.assertEqual('incorrect license', next(errors)[2])

    def test_rust(self):
        errors = tidy.collect_errors_for_files(iterFile('rust_tidy.rs'), [], [tidy.check_rust], print_text=False)
        self.assertTrue('mod declaration is not in alphabetical order' in next(errors)[2])
        self.assertEqual('mod declaration spans multiple lines', next(errors)[2])
        self.assertTrue('derivable traits list is not in alphabetical order' in next(errors)[2])
        self.assertEqual('found an empty line following a {', next(errors)[2])
        self.assertEqual('use &[T] instead of &Vec<T>', next(errors)[2])
        self.assertEqual('use &str instead of &String', next(errors)[2])
        self.assertEqual('use &T instead of &Root<T>', next(errors)[2])
        self.assertEqual('use &T instead of &DomRoot<T>', next(errors)[2])
        self.assertEqual('encountered function signature with -> ()', next(errors)[2])
        self.assertEqual('operators should go at the end of the first line', next(errors)[2])
        self.assertEqual('unwrap() or panic!() found in code which should not panic.', next(errors)[2])
        self.assertEqual('unwrap() or panic!() found in code which should not panic.', next(errors)[2])
        self.assertNoMoreErrors(errors)

        feature_errors = tidy.collect_errors_for_files(iterFile('lib.rs'), [], [tidy.check_rust], print_text=False)

        self.assertTrue('feature attribute is not in alphabetical order' in next(feature_errors)[2])
        self.assertTrue('feature attribute is not in alphabetical order' in next(feature_errors)[2])
        self.assertTrue('feature attribute is not in alphabetical order' in next(feature_errors)[2])
        self.assertTrue('feature attribute is not in alphabetical order' in next(feature_errors)[2])
        self.assertNoMoreErrors(feature_errors)

        ban_errors = tidy.collect_errors_for_files(iterFile('ban.rs'), [], [tidy.check_rust], print_text=False)
        self.assertEqual('Banned type Cell<JSVal> detected. Use MutDom<JSVal> instead', next(ban_errors)[2])
        self.assertNoMoreErrors(ban_errors)

        ban_errors = tidy.collect_errors_for_files(iterFile('ban-domrefcell.rs'), [], [tidy.check_rust], print_text=False)
        self.assertEqual('Banned type DomRefCell<Dom<T>> detected. Use MutDom<T> instead', next(ban_errors)[2])
        self.assertNoMoreErrors(ban_errors)

    def test_spec_link(self):
        tidy.SPEC_BASE_PATH = BASE_PATH
        errors = tidy.collect_errors_for_files(iterFile('speclink.rs'), [], [tidy.check_spec], print_text=False)
        self.assertEqual('method declared in webidl is missing a comment with a specification link', next(errors)[2])
        self.assertEqual('method declared in webidl is missing a comment with a specification link', next(errors)[2])
        self.assertNoMoreErrors(errors)

    def test_webidl(self):
        errors = tidy.collect_errors_for_files(iterFile('spec.webidl'), [tidy.check_webidl_spec], [], print_text=False)
        self.assertEqual('No specification link found.', next(errors)[2])
        self.assertNoMoreErrors(errors)

    def test_toml(self):
        errors = tidy.collect_errors_for_files(iterFile('Cargo.toml'), [], [tidy.check_toml], print_text=False)
        self.assertEqual('found asterisk instead of minimum version number', next(errors)[2])
        self.assertEqual('.toml file should contain a valid license.', next(errors)[2])
        self.assertNoMoreErrors(errors)

    def test_modeline(self):
        errors = tidy.collect_errors_for_files(iterFile('modeline.txt'), [], [tidy.check_modeline], print_text=False)
        self.assertEqual('vi modeline present', next(errors)[2])
        self.assertEqual('vi modeline present', next(errors)[2])
        self.assertEqual('vi modeline present', next(errors)[2])
        self.assertEqual('emacs file variables present', next(errors)[2])
        self.assertEqual('emacs file variables present', next(errors)[2])
        self.assertNoMoreErrors(errors)

    def test_malformed_json(self):
        errors = tidy.collect_errors_for_files(iterFile('malformed_json.json'), [tidy.check_json], [], print_text=False)
        self.assertEqual('Invalid control character at: line 3 column 40 (char 61)', next(errors)[2])
        self.assertNoMoreErrors(errors)

    def test_json_with_duplicate_key(self):
        errors = tidy.collect_errors_for_files(iterFile('duplicate_key.json'), [tidy.check_json], [], print_text=False)
        self.assertEqual('Duplicated Key (the_duplicated_key)', next(errors)[2])
        self.assertNoMoreErrors(errors)

    def test_json_with_unordered_keys(self):
        tidy.config["check-ordered-json-keys"].append('python/tidy/tests/unordered_key.json')
        errors = tidy.collect_errors_for_files(iterFile('unordered_key.json'), [tidy.check_json], [], print_text=False)
        self.assertEqual('Unordered key (found b before a)', next(errors)[2])
        self.assertNoMoreErrors(errors)

    def test_lock(self):
        errors = tidy.run_custom_cargo_lock_lints(test_file_path('duplicated_package.lock'), print_text=False)
        msg = """duplicate versions for package `test`
\t\x1b[93mThe following packages depend on version 0.4.9 from 'crates.io':\x1b[0m
\t\ttest2 0.1.0
\t\x1b[93mThe following packages depend on version 0.5.1 from 'crates.io':\x1b[0m
\t\ttest3 0.5.1"""
        self.assertEqual(msg, next(errors)[2])
        msg2 = """duplicate versions for package `test3`
\t\x1b[93mThe following packages depend on version 0.5.1 from 'crates.io':\x1b[0m
\t\ttest4 0.1.0
\t\x1b[93mThe following packages depend on version 0.5.1 from 'https://github.com/user/test3':\x1b[0m
\t\ttest5 0.1.0"""
        self.assertEqual(msg2, next(errors)[2])
        self.assertNoMoreErrors(errors)

    def test_lock_ignore_without_duplicates(self):
        tidy.config["ignore"]["packages"] = ["test", "test2", "test3", "test5"]
        errors = tidy.run_custom_cargo_lock_lints(test_file_path('duplicated_package.lock'), print_text=False)

        msg = (
            "duplicates for `test2` are allowed, but only single version found"
            "\n\t\x1b[93mThe following packages depend on version 0.1.0 from 'https://github.com/user/test2':\x1b[0m"
        )
        self.assertEqual(msg, next(errors)[2])

        msg2 = (
            "duplicates for `test5` are allowed, but only single version found"
            "\n\t\x1b[93mThe following packages depend on version 0.1.0 from 'https://github.com/':\x1b[0m"
        )
        self.assertEqual(msg2, next(errors)[2])

        self.assertNoMoreErrors(errors)

    def test_lock_exceptions(self):
        tidy.config["blocked-packages"]["rand"] = ["test_exception", "test_unneeded_exception"]
        errors = tidy.run_custom_cargo_lock_lints(test_file_path('blocked_package.lock'), print_text=False)

        msg = (
            "Package test_blocked 0.0.2 depends on blocked package rand."
        )

        msg2 = (
            "Package test_unneeded_exception is not required to be an exception of blocked package rand."
        )

        self.assertEqual(msg, next(errors)[2])
        self.assertEqual(msg2, next(errors)[2])
        self.assertNoMoreErrors(errors)

        # needed to not raise errors in other test cases
        tidy.config["blocked-packages"]["rand"] = []

    def test_file_list(self):
        file_path = os.path.join(BASE_PATH, 'test_ignored')
        file_list = tidy.FileList(file_path, only_changed_files=False, exclude_dirs=[], progress=False)
        lst = list(file_list)
        self.assertEqual([os.path.join(file_path, 'whee', 'test.rs'), os.path.join(file_path, 'whee', 'foo', 'bar.rs')], lst)
        file_list = tidy.FileList(file_path, only_changed_files=False,
                                  exclude_dirs=[os.path.join(file_path, 'whee', 'foo')],
                                  progress=False)
        lst = list(file_list)
        self.assertEqual([os.path.join(file_path, 'whee', 'test.rs')], lst)

    def test_multiline_string(self):
        errors = tidy.collect_errors_for_files(iterFile('multiline_string.rs'), [], [tidy.check_rust], print_text=False)
        self.assertNoMoreErrors(errors)


def run_tests():
    verbosity = 1 if logging.getLogger().level >= logging.WARN else 2
    suite = unittest.TestLoader().loadTestsFromTestCase(CheckTidiness)
    return unittest.TextTestRunner(verbosity=verbosity).run(suite).wasSuccessful()
