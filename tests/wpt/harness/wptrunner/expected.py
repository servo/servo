# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import os


def expected_path(metadata_path, test_path):
    """Path to the expectation data file for a given test path.

    This is defined as metadata_path + relative_test_path + .ini

    :param metadata_path: Path to the root of the metadata directory
    :param test_path: Relative path to the test file from the test root
    """
    args = list(test_path.split("/"))
    args[-1] += ".ini"
    return os.path.join(metadata_path, *args)
