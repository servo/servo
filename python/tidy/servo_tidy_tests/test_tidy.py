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

    def test_tidy_config(self):
        errors = tidy.check_config_file(os.path.join(base_path, 'servo-tidy.toml'), print_text=False)
        self.assertEqual("invalid config key 'key-outside'", errors.next()[2])
        self.assertEqual("invalid config key 'wrong-key'", errors.next()[2])
        self.assertEqual('invalid config table [wrong]', errors.next()[2])
        self.assertEqual("ignored file './fake/file.html' doesn't exist", errors.next()[2])
        self.assertEqual("ignored directory './fake/dir' doesn't exist", errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_directory_checks(self):
        dirs = {
            os.path.join(base_path, "dir_check/webidl_plus"): ['webidl', 'test'],
            os.path.join(base_path, "dir_check/only_webidl"): ['webidl']
            }
        errors = tidy.check_directory_files(dirs)
        error_dir = os.path.join(base_path, "dir_check/webidl_plus")
        self.assertEqual("Unexpected extension found for test.rs. We only expect files with webidl, test extensions in {0}".format(error_dir), errors.next()[2])
        self.assertEqual("Unexpected extension found for test2.rs. We only expect files with webidl, test extensions in {0}".format(error_dir), errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_spaces_correctnes(self):
        errors = tidy.collect_errors_for_files(iterFile('wrong_space.rs'), [], [tidy.check_by_line], print_text=False)
        self.assertEqual('trailing whitespace', errors.next()[2])
        self.assertEqual('no newline at EOF', errors.next()[2])
        self.assertEqual('tab on line', errors.next()[2])
        self.assertEqual('CR on line', errors.next()[2])
        self.assertEqual('no newline at EOF', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_empty_file(self):
        errors = tidy.collect_errors_for_files(iterFile('empty_file.rs'), [], [tidy.check_by_line], print_text=False)
        self.assertEqual('file is empty', errors.next()[2])
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

    def test_license(self):
        errors = tidy.collect_errors_for_files(iterFile('incorrect_license.rs'), [], [tidy.check_license], print_text=False)
        self.assertEqual('incorrect license', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_shebang_license(self):
        errors = tidy.collect_errors_for_files(iterFile('shebang_license.py'), [], [tidy.check_license], print_text=False)
        self.assertEqual('missing blank line after shebang', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_shell(self):
        errors = tidy.collect_errors_for_files(iterFile('shell_tidy.sh'), [], [tidy.check_shell], print_text=False)
        self.assertEqual('script does not have shebang "#!/usr/bin/env bash"', errors.next()[2])
        self.assertEqual('script is missing options "set -o errexit", "set -o pipefail"', errors.next()[2])
        self.assertEqual('script should not use backticks for command substitution', errors.next()[2])
        self.assertEqual('variable substitutions should use the full \"${VAR}\" form', errors.next()[2])
        self.assertEqual('script should use `[[` instead of `[` for conditional testing', errors.next()[2])
        self.assertEqual('script should use `[[` instead of `[` for conditional testing', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_apache2_incomplete(self):
        errors = tidy.collect_errors_for_files(iterFile('apache2_license.rs'), [], [tidy.check_license])
        self.assertEqual('incorrect license', errors.next()[2])

    def test_rust(self):
        errors = tidy.collect_errors_for_files(iterFile('rust_tidy.rs'), [], [tidy.check_rust], print_text=False)
        self.assertEqual('extra space after use', errors.next()[2])
        self.assertEqual('extra space after {', errors.next()[2])
        self.assertEqual('extra space before }', errors.next()[2])
        self.assertEqual('use statement spans multiple lines', errors.next()[2])
        self.assertEqual('missing space before }', errors.next()[2])
        self.assertTrue('use statement is not in alphabetical order' in errors.next()[2])
        self.assertEqual('use statement contains braces for single import', errors.next()[2])
        self.assertTrue('use statement is not in alphabetical order' in errors.next()[2])
        self.assertEqual('encountered whitespace following a use statement', errors.next()[2])
        self.assertTrue('mod declaration is not in alphabetical order' in errors.next()[2])
        self.assertEqual('mod declaration spans multiple lines', errors.next()[2])
        self.assertTrue('extern crate declaration is not in alphabetical order' in errors.next()[2])
        self.assertTrue('derivable traits list is not in alphabetical order' in errors.next()[2])
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
        self.assertEqual('encountered function signature with -> ()', errors.next()[2])
        self.assertEqual('operators should go at the end of the first line', errors.next()[2])
        self.assertEqual('else braces should be on the same line', errors.next()[2])
        self.assertEqual('extra space after (', errors.next()[2])
        self.assertEqual('extra space after (', errors.next()[2])
        self.assertEqual('extra space after (', errors.next()[2])
        self.assertEqual('extra space after test_fun', errors.next()[2])
        self.assertEqual('no = in the beginning of line', errors.next()[2])
        self.assertEqual('space before { is not a multiple of 4', errors.next()[2])
        self.assertEqual('space before } is not a multiple of 4', errors.next()[2])
        self.assertEqual('extra space after if', errors.next()[2])
        self.assertNoMoreErrors(errors)

        feature_errors = tidy.collect_errors_for_files(iterFile('lib.rs'), [], [tidy.check_rust], print_text=False)

        self.assertTrue('feature attribute is not in alphabetical order' in feature_errors.next()[2])
        self.assertTrue('feature attribute is not in alphabetical order' in feature_errors.next()[2])
        self.assertTrue('feature attribute is not in alphabetical order' in feature_errors.next()[2])
        self.assertTrue('feature attribute is not in alphabetical order' in feature_errors.next()[2])
        self.assertNoMoreErrors(feature_errors)

        ban_errors = tidy.collect_errors_for_files(iterFile('ban.rs'), [], [tidy.check_rust], print_text=False)
        self.assertEqual('Banned type Cell<JSVal> detected. Use MutDom<JSVal> instead', ban_errors.next()[2])
        self.assertNoMoreErrors(ban_errors)

        ban_errors = tidy.collect_errors_for_files(iterFile('ban-domrefcell.rs'), [], [tidy.check_rust], print_text=False)
        self.assertEqual('Banned type DomRefCell<Dom<T>> detected. Use MutDom<T> instead', ban_errors.next()[2])
        self.assertNoMoreErrors(ban_errors)

    def test_spec_link(self):
        tidy.SPEC_BASE_PATH = base_path
        errors = tidy.collect_errors_for_files(iterFile('speclink.rs'), [], [tidy.check_spec], print_text=False)
        self.assertEqual('method declared in webidl is missing a comment with a specification link', errors.next()[2])
        self.assertEqual('method declared in webidl is missing a comment with a specification link', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_script_thread(self):
        errors = tidy.collect_errors_for_files(iterFile('script_thread.rs'), [], [tidy.check_rust], print_text=False)
        self.assertEqual('use a separate variable for the match expression', errors.next()[2])
        self.assertEqual('use a separate variable for the match expression', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_webidl(self):
        errors = tidy.collect_errors_for_files(iterFile('spec.webidl'), [tidy.check_webidl_spec], [], print_text=False)
        self.assertEqual('No specification link found.', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_toml(self):
        errors = tidy.collect_errors_for_files(iterFile('Cargo.toml'), [tidy.check_toml], [], print_text=False)
        self.assertEqual('found asterisk instead of minimum version number', errors.next()[2])
        self.assertEqual('.toml file should contain a valid license.', errors.next()[2])
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

    def test_json_with_unordered_keys(self):
        tidy.config["check-ordered-json-keys"].append('python/tidy/servo_tidy_tests/unordered_key.json')
        errors = tidy.collect_errors_for_files(iterFile('unordered_key.json'), [tidy.check_json], [], print_text=False)
        self.assertEqual('Unordered key (found b before a)', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_yaml_with_duplicate_key(self):
        errors = tidy.collect_errors_for_files(iterFile('duplicate_keys_buildbot_steps.yml'), [tidy.check_yaml], [], print_text=False)
        self.assertEqual('Duplicated Key (duplicate_yaml_key)', errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_non_list_mapped_buildbot_steps(self):
        errors = tidy.collect_errors_for_files(iterFile('non_list_mapping_buildbot_steps.yml'), [tidy.check_yaml], [], print_text=False)
        self.assertEqual("expected a list for dictionary value @ data['non-list-key']", errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_non_string_list_mapping_buildbot_steps(self):
        errors = tidy.collect_errors_for_files(iterFile('non_string_list_buildbot_steps.yml'), [tidy.check_yaml], [], print_text=False)
        self.assertEqual("expected str @ data['mapping_key'][0]", errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_lock(self):
        errors = tidy.collect_errors_for_files(iterFile('duplicated_package.lock'), [tidy.check_lock], [], print_text=False)
        msg = """duplicate versions for package `test`
\t\x1b[93mThe following packages depend on version 0.4.9 from 'crates.io':\x1b[0m
\t\ttest2
\t\x1b[93mThe following packages depend on version 0.5.1 from 'crates.io':\x1b[0m"""
        self.assertEqual(msg, errors.next()[2])
        msg2 = """duplicate versions for package `test3`
\t\x1b[93mThe following packages depend on version 0.5.1 from 'crates.io':\x1b[0m
\t\ttest4
\t\x1b[93mThe following packages depend on version 0.5.1 from 'https://github.com/user/test3':\x1b[0m
\t\ttest5"""
        self.assertEqual(msg2, errors.next()[2])
        self.assertNoMoreErrors(errors)

    def test_lock_ignore_without_duplicates(self):
        tidy.config["ignore"]["packages"] = ["test", "test2", "test3", "test5"]
        errors = tidy.collect_errors_for_files(iterFile('duplicated_package.lock'), [tidy.check_lock], [], print_text=False)

        msg = (
            "duplicates for `test2` are allowed, but only single version found"
            "\n\t\x1b[93mThe following packages depend on version 0.1.0 from 'https://github.com/user/test2':\x1b[0m"
        )
        self.assertEqual(msg, errors.next()[2])

        msg2 = (
            "duplicates for `test5` are allowed, but only single version found"
            "\n\t\x1b[93mThe following packages depend on version 0.1.0 from 'https://github.com/':\x1b[0m"
        )
        self.assertEqual(msg2, errors.next()[2])

        self.assertNoMoreErrors(errors)

    def test_lint_runner(self):
        test_path = base_path + 'lints/'
        runner = tidy.LintRunner(only_changed_files=False, progress=False)
        runner.path = test_path + 'some-fictional-file'
        self.assertEqual([(runner.path, 0, "file does not exist")], list(runner.check()))
        runner.path = test_path + 'not_script'
        self.assertEqual([(runner.path, 0, "lint should be a python script")],
                         list(runner.check()))
        runner.path = test_path + 'not_inherited.py'
        self.assertEqual([(runner.path, 1, "class 'Lint' should inherit from 'LintRunner'")],
                         list(runner.check()))
        runner.path = test_path + 'no_lint.py'
        self.assertEqual([(runner.path, 1, "script should contain a class named 'Lint'")],
                         list(runner.check()))
        runner.path = test_path + 'no_run.py'
        self.assertEqual([(runner.path, 0, "class 'Lint' should implement 'run' method")],
                         list(runner.check()))
        runner.path = test_path + 'invalid_error_tuple.py'
        self.assertEqual([(runner.path, 1, "errors should be a tuple of (path, line, reason)")],
                         list(runner.check()))
        runner.path = test_path + 'proper_file.py'
        self.assertEqual([('path', 0, "foobar")], list(runner.check()))

    def test_file_list(self):
        base_path='./python/tidy/servo_tidy_tests/test_ignored'
        file_list = tidy.FileList(base_path, only_changed_files=False, exclude_dirs=[])
        lst = list(file_list)
        self.assertEqual([os.path.join(base_path, 'whee', 'test.rs'), os.path.join(base_path, 'whee', 'foo', 'bar.rs')], lst)
        file_list = tidy.FileList(base_path, only_changed_files=False,
                                  exclude_dirs=[os.path.join(base_path, 'whee', 'foo')])
        lst = list(file_list)
        self.assertEqual([os.path.join(base_path, 'whee', 'test.rs')], lst)

    def test_multiline_string(self):
        errors = tidy.collect_errors_for_files(iterFile('multiline_string.rs'), [], [tidy.check_rust], print_text=True)
        self.assertNoMoreErrors(errors)


def do_tests():
    suite = unittest.TestLoader().loadTestsFromTestCase(CheckTidiness)
    return 0 if unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful() else 1
