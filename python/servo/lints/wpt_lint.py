# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import os
import sys

from servo_tidy.tidy import LintRunner, filter_file

WPT_PATH = os.path.join(".", "tests", "wpt")
SUITES = ["web-platform-tests", os.path.join("mozilla", "tests")]


class Lint(LintRunner):
    def _get_wpt_files(self, suite):
        working_dir = os.path.join(WPT_PATH, suite, '')
        file_iter = self.get_files(working_dir, exclude_dirs=[])
        print '\nRunning the WPT lint on %s...' % working_dir
        for f in file_iter:
            if filter_file(f):
                yield f[len(working_dir):]

    def run(self):
        if self.stylo:
            return

        wpt_working_dir = os.path.abspath(os.path.join(WPT_PATH, "web-platform-tests"))
        for suite in SUITES:
            files = self._get_wpt_files(suite)
            sys.path.insert(0, wpt_working_dir)
            from tools.lint import lint
            sys.path.remove(wpt_working_dir)
            file_dir = os.path.abspath(os.path.join(WPT_PATH, suite))
            returncode = lint.lint(file_dir, list(files), output_format="json")
            if returncode:
                yield ("WPT Lint Tool", "", "lint error(s) in Web Platform Tests: exit status %s" % returncode)
