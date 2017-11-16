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
import random
from enum import Enum
DEVNULL = open(os.devnull, 'wb')


class Status(Enum):
    KILLED = 0
    SURVIVED = 1
    SKIPPED = 2
    UNEXPECTED = 3


def mutate_random_line(file_name, strategy):
    line_numbers = []
    for line in fileinput.input(file_name):
        if re.search(strategy['regex'], line):
            line_numbers.append(fileinput.lineno())
    if len(line_numbers) == 0:
        return -1
    else:
        mutation_line_number = line_numbers[random.randint(0, len(line_numbers) - 1)]
        for line in fileinput.input(file_name, inplace=True):
            if fileinput.lineno() == mutation_line_number:
                line = re.sub(strategy['regex'], strategy['replaceString'], line)
            print line.rstrip()
        return mutation_line_number


def mutation_test(file_name, tests):
    status = Status.UNEXPECTED
    local_changes_present = subprocess.call('git diff --quiet {0}'.format(file_name), shell=True)
    if local_changes_present == 1:
        status = Status.SKIPPED
        print "{0} has local changes, please commit/remove changes before running the test".format(file_name)
    else:
        strategy = {'regex': r'\s&&\s', 'replaceString': ' || '}
        mutated_line = mutate_random_line(file_name, strategy)
        if mutated_line != -1:
            print "Mutating {0} at line {1}".format(file_name, mutated_line)
            print "compiling mutant {0}:{1}".format(file_name, mutated_line)
            sys.stdout.flush()
            subprocess.call('python mach build --release', shell=True, stdout=DEVNULL)
            for test in tests:
                test_command = "python mach test-wpt {0} --release".format(test.encode('utf-8'))
                print "running `{0}` test for mutant {1}:{2}".format(test, file_name, mutated_line)
                sys.stdout.flush()
                test_status = subprocess.call(test_command, shell=True, stdout=DEVNULL)
                if test_status != 0:
                    print("Failed: while running `{0}`".format(test_command))
                    print "mutated file {0} diff".format(file_name)
                    sys.stdout.flush()
                    subprocess.call('git --no-pager diff {0}'.format(file_name), shell=True)
                    status = Status.SURVIVED
                else:
                    print("Success: Mutation killed by {0}".format(test.encode('utf-8')))
                    status = Status.KILLED
                    break
            print "reverting mutant {0}:{1}".format(file_name, mutated_line)
            sys.stdout.flush()
            subprocess.call('git checkout {0}'.format(file_name), shell=True)
        else:
            print "Cannot mutate {0}".format(file_name)
        print "-" * 80 + "\n"
    return status
