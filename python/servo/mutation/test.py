# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import subprocess
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
        strategies = list(get_strategies())
        while len(strategies):
            strategy = random.choice(strategies)
            strategies.remove(strategy)
            mutator = Mutator(strategy())
            mutated_line = mutator.mutate(file_name)
            if mutated_line != -1:
                logging.info("Mutated {0} at line {1}".format(file_name, mutated_line))
                logging.info("compiling mutant {0}:{1}".format(file_name, mutated_line))
                test_command = "python mach build --release"
                if subprocess.call(test_command, shell=True, stdout=DEVNULL):
                    logging.error("Compilation Failed: Unexpected error")
                    logging.error("Failed: while running `{0}`".format(test_command))
                    subprocess.call('git --no-pager diff {0}'.format(file_name), shell=True)
                    status = Status.UNEXPECTED
                else:
                    for test in tests:
                        test_command = "python mach test-wpt {0} --release".format(test.encode('utf-8'))
                        logging.info("running `{0}` test for mutant {1}:{2}".format(test, file_name, mutated_line))
                        test_status = subprocess.call(test_command, shell=True, stdout=DEVNULL)
                        if test_status != 0:
                            logging.error("Failed: while running `{0}`".format(test_command))
                            logging.error("mutated file {0} diff".format(file_name))
                            subprocess.call('git --no-pager diff {0}'.format(file_name), shell=True)
                            status = Status.SURVIVED
                        else:
                            logging.info("Success: Mutation killed by {0}".format(test.encode('utf-8')))
                            status = Status.KILLED
                            break
                logging.info("reverting mutant {0}:{1}\n".format(file_name, mutated_line))
                subprocess.call('git checkout {0}'.format(file_name), shell=True)
                break
            elif not len(strategies):
                # All strategies are tried
                logging.info("\nCannot mutate {0}\n".format(file_name))
    return status
