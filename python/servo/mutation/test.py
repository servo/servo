# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import fileinput
import re
import subprocess
import sys
import os
DEVNULL = open(os.devnull, 'wb')


def mutate_line(file_name, line_number):
    for line in fileinput.input(file_name, inplace=True):
        if fileinput.lineno() == line_number:
            line = re.sub(r'\s&&\s', ' || ', line)
        print line.rstrip()


def mutation_test(file_name, tests):
    line_numbers = []
    for line in fileinput.input(file_name):
        if re.search(r'\s&&\s', line):
            line_numbers.append(fileinput.lineno())

    for line_to_mutate in line_numbers:
        print "Mutating {0} at line {1}".format(file_name, line_to_mutate)
        mutate_line(file_name, line_to_mutate)
        print "compling mutant {0}:{1}".format(file_name, line_to_mutate)
        sys.stdout.flush()
        subprocess.call('python mach build --release', shell=True, stdout=DEVNULL)
        print "running tests for mutant {0}:{1}".format(file_name, line_to_mutate)
        sys.stdout.flush()
        for test in tests:
            test_command = "python mach test-wpt {0} --release".format(test.encode('utf-8'))
            test_status = subprocess.call(test_command, shell=True, stdout=DEVNULL)
            if test_status != 0:
                print("Failed: while running `{0}`".format(test_command))
                print "mutated file {0} diff".format(file_name)
                sys.stdout.flush()
                subprocess.call('git --no-pager diff {0}'.format(file_name), shell=True)
            else:
                print("Success: Mutation killed by {0}".format(test.encode('utf-8')))
                break
        print "reverting mutant {0}:{1}".format(file_name, line_to_mutate)
        sys.stdout.flush()
        subprocess.call('git checkout {0}'.format(file_name), shell=True)
