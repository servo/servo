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


class Mutator:
    def __init__(self, strategy):
        self._strategy = strategy

    def mutate(self, file_name):
        return self._strategy.mutate_random_line(file_name)


class Strategy:
    def __init__(self):
        self._replace_strategy = {}

    def mutate_random_line(self, file_name):
        line_numbers = []
        for line in fileinput.input(file_name):
            if re.search(self._replace_strategy['regex'], line):
                line_numbers.append(fileinput.lineno())
        if len(line_numbers) == 0:
            return -1
        else:
            mutation_line_number = line_numbers[random.randint(0, len(line_numbers) - 1)]
            for line in fileinput.input(file_name, inplace=True):
                if fileinput.lineno() == mutation_line_number:
                    line = re.sub(self._replace_strategy['regex'], self._replace_strategy['replaceString'], line)
                print line.rstrip()
            return mutation_line_number


class AndOr(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        self._replace_strategy = {
            'regex': r'\s&&\s',
            'replaceString': ' || '
        }


class IfTrue(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        self._replace_strategy = {
            'regex': r'(?<=if\s)(.*)(?=\s\{)',
            'replaceString': 'true'
        }


class IfFalse(Strategy):
    def __init__(self):
        Strategy.__init__(self)
        self._replace_strategy = {
            'regex': r'(?<=if\s)(.*)(?=\s\{)',
            'replaceString': 'false'
        }


def mutation_test(file_name, tests):
    status = Status.UNEXPECTED
    local_changes_present = subprocess.call('git diff --quiet {0}'.format(file_name), shell=True)
    if local_changes_present == 1:
        status = Status.SKIPPED
        print "{0} has local changes, please commit/remove changes before running the test".format(file_name)
    else:
        mutation_strategies = (AndOr, IfTrue, IfFalse)
        strategy = random.choice(mutation_strategies)()
        mutator = Mutator(strategy)
        mutated_line = mutator.mutate(file_name)
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
