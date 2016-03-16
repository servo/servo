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
from cStringIO import StringIO
import re


class Capturing(list):
    def __enter__(self):
        self._stdout = sys.stdout
        sys.stdout = self._stringio = StringIO()
        return self

    def __exit__(self, *args):
        self.extend(self._stringio.getvalue().splitlines())
        sys.stdout = self._stdout


class CheckTidiness(unittest.TestCase):
    @classmethod
    def setUpClass(self):
        # group messages into dict with file name as a key
        self.results = dict()
        prog = re.compile(r"(\w+\.(rs|webidl))")    # file match pattern
        with Capturing() as output:
            tidy.scan(False, True, './python/tidy_self_test/')
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

    def test_long_line(self):
        self.assertTrue('long_line.rs' in self.results)
        output = self.results['long_line.rs']
        self.assertTrue('Line is longer than 120 characters' in output[0])

    def test_whatwg_link(self):
        self.assertTrue('whatwg_link.rs' in self.results)
        output = self.results['whatwg_link.rs']
        self.assertTrue('link to WHATWG may break in the future, use this format instead:' in output[0])
        self.assertTrue('links to WHATWG single-page url, change to multi page:' in output[1])

    def test_rust(self):
        self.assertTrue('rust_tidy.rs' in self.results)
        output = self.results['rust_tidy.rs']
        self.assertTrue('use statement spans multiple lines' in output[0])
        self.assertTrue('missing space before }' in output[1])
        self.assertTrue('use statement is not in alphabetical order' in output[2])
        self.assertTrue('missing space before ->' in output[3])
        self.assertTrue('missing space after ->' in output[4])
        self.assertTrue('missing space after :' in output[5])
        self.assertTrue('missing space before {' in output[6])
        self.assertTrue('missing space before =' in output[7])
        self.assertTrue('missing space after =' in output[8])
        self.assertTrue('missing space before -' in output[9])
        self.assertTrue('missing space before *' in output[10])
        self.assertTrue('missing space after =>' in output[11])
        self.assertTrue('extra space before :' in output[12])
        self.assertTrue('extra space before :' in output[13])
        self.assertTrue('use &[T] instead of &Vec<T>' in output[14])
        self.assertTrue('use &str instead of &String' in output[15])
        # TODO missing tests for:
        # extern crate declaration
        # encountered whitespace following a use statement
        # mod declaration spans multiple lines

    def test_webidl(self):
        self.assertTrue('spec.webidl' in self.results)
        output = self.results['spec.webidl']
        self.assertTrue('No specification link found.' in output[0])


def print_usage():
    print("USAGE: python {0}".format(sys.argv[0]))


def doTests():
    suite = unittest.TestLoader().loadTestsFromTestCase(CheckTidiness)
    unittest.TextTestRunner(verbosity=2).run(suite)
