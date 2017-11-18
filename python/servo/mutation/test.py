# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import subprocess
import sys
import os
import random
import logging

from mutator import Mutator, get_strategies
from enum import Enum
DEVNULL = open(os.devnull, 'wb')

logging.basicConfig(format="%(asctime)s %(levelname)s: %(message)s", level=logging.DEBUG)


class Status(Enum):
    KILLED = 0
    SURVIVED = 1
    SKIPPED = 2
    UNEXPECTED = 3


def mutation_test(file_name, tests):
    status = Status.UNEXPECTED
    local_changes_present = subprocess.call('git diff --quiet {0}'.format(file_name), shell=True)
    if local_changes_present == 1:
        status = Status.SKIPPED
        logging.warning("{0} has local changes, please commit/remove changes before running the test".format(file_name))
    else:
        strategy = random.choice(get_strategies())()
        mutator = Mutator(strategy)
        mutated_line = mutator.mutate(file_name)
        if mutated_line != -1:
            logging.info("Mutated {0} at line {1}".format(file_name, mutated_line))
            logging.info("compiling mutant {0}:{1}".format(file_name, mutated_line))
            sys.stdout.flush()
            if subprocess.call('python mach build --release', shell=True, stdout=DEVNULL):
                logging.error("Compilation Failed: Unexpected error")
                status = Status.UNEXPECTED
            else:
                for test in tests:
                    test_command = "python mach test-wpt {0} --release".format(test.encode('utf-8'))
                    logging.info("running `{0}` test for mutant {1}:{2}".format(test, file_name, mutated_line))
                    sys.stdout.flush()
                    test_status = subprocess.call(test_command, shell=True, stdout=DEVNULL)
                    if test_status != 0:
                        logging.error("Failed: while running `{0}`".format(test_command))
                        logging.error("mutated file {0} diff".format(file_name))
                        sys.stdout.flush()
                        subprocess.call('git --no-pager diff {0}'.format(file_name), shell=True)
                        status = Status.SURVIVED
                    else:
                        logging.info("Success: Mutation killed by {0}".format(test.encode('utf-8')))
                        status = Status.KILLED
                        break
                logging.info("reverting mutant {0}:{1}".format(file_name, mutated_line))
            sys.stdout.flush()
            subprocess.call('git checkout {0}'.format(file_name), shell=True)
        else:
            logging.info("Cannot mutate {0}".format(file_name))
        print "-" * 80 + "\n"
    return status
