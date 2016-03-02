# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import unittest

# import tidy from parent directory
import os
import sys
sys.path.insert(1, os.path.join(sys.path[0], '..'))
import tidy
import mach_bootstrap
from cStringIO import StringIO
import sys
import re


class Capturing(list):
    def __enter__(self):
        self._stdout = sys.stdout
        sys.stdout = self._stringio = StringIO()
        return self

    def __exit__(self, *args):
        self.extend(self._stringio.getvalue().splitlines())
        sys.stdout = self._stdout


class CheckRustTidiness(unittest.TestCase):
    @classmethod
    def setUpClass(self):
        # group messages into dict with file name as a key
        self.results = dict()
        prog = re.compile(r"(\w+\.(rs|py))")    # file match pattern
        with Capturing() as output:
            tidy.scan(False)
        for o in output:
            match = prog.search(o)
            if match:
                fileName = match.group(0)
                if fileName not in self.results:
                    self.results[fileName] = []
                self.results[fileName].append(o)

    def test_spaces_correctnes(self):
        self.assertTrue('wrong_space.rs' in self.results)
        output = self.results['wrong_space.rs']
        self.assertTrue('trailing whitespace' in output[0])
        self.assertTrue('tab on line' in output[2])
        self.assertTrue('CR on line' in output[3])
        self.assertTrue('no newline at EOF' in output[4])

    def test_licence(self):
        self.assertTrue('incorrect_license.rs' in self.results)
        output = self.results['incorrect_license.rs']
        self.assertTrue('incorrect license' in output[0])


def print_usage():
    print("USAGE: python {0}".format(sys.argv[0]))


if __name__ == '__main__':
    if len(sys.argv) > 1:
        print_usage()
    else:
        mach_bootstrap._activate_virtualenv('../../')
        unittest.main()
