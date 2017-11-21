# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from os import listdir
from os.path import isfile, isdir, join
import json
import sys
import test
import logging
import random

test_summary = {
    test.Status.KILLED: 0,
    test.Status.SURVIVED: 0,
    test.Status.SKIPPED: 0,
    test.Status.UNEXPECTED: 0
}


def get_folders_list(path):
    folder_list = []
    for filename in listdir(path):
        if isdir(join(path, filename)):
            folder_name = join(path, filename)
            folder_list.append(folder_name)
        return folder_list


def mutation_test_for(mutation_path):
    test_mapping_file = join(mutation_path, 'test_mapping.json')
    if isfile(test_mapping_file):
        json_data = open(test_mapping_file).read()
        test_mapping = json.loads(json_data)
        # Run mutation test for all source files in mapping file in a random order.
        source_files = list(test_mapping.keys())
        random.shuffle(source_files)
        for src_file in source_files:
            status = test.mutation_test(join(mutation_path, src_file.encode('utf-8')), test_mapping[src_file])
            test_summary[status] += 1
        # Run mutation test in all folder in the path.
        for folder in get_folders_list(mutation_path):
            mutation_test_for(folder)
    else:
        logging.warning("This folder {0} has no test mapping file.".format(mutation_path))


mutation_test_for(sys.argv[1])
logging.basicConfig(format="%(asctime)s %(levelname)s: %(message)s", level=logging.DEBUG)
logging.info("Test Summary:")
logging.info("Mutant Killed (Success) \t\t{0}".format(test_summary[test.Status.KILLED]))
logging.info("Mutant Survived (Failure) \t{0}".format(test_summary[test.Status.SURVIVED]))
logging.info("Mutation Skipped \t\t\t{0}".format(test_summary[test.Status.SKIPPED]))
logging.info("Unexpected error in mutation \t{0}".format(test_summary[test.Status.UNEXPECTED]))
if test_summary[test.Status.SURVIVED]:
    sys.exit(1)
