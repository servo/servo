# -*- coding: utf-8 -*-
"""
create_test_output.py
~~~~~~~~~~~~~~~~~~~~~

This script uses the Python HPACK library to take the raw data input used in
https://github.com/http2jp/hpack-test-case and produce appropriately formatted
output for that repository. This makes it possible to keep the Python HPACK
library information in the hpack-test-case repository up to date.

For many reasons, this script is not distributed with the library itself.
Instead, it is kept in the repository, and expects to be run under tox.

It should be invoked as follows:

    $ python create_test_output.py /path/to/hpack-test-case/repository
"""
from __future__ import print_function, absolute_import

import binascii
import codecs
import json
import os
import os.path
import sys

import hpack


def headers_from_story(story_headers):
    """
    The story provides headers as a list of JSON objects. That's strongly not
    what we want: we want a list of two-tuples. This function transforms
    from the list of JSON objects to a list of two-tuples.
    """
    useful_headers = []
    for pair in story_headers:
        assert len(pair) == 1
        useful_headers.extend(list(pair.items()))

    return useful_headers


def process_story(story_name, raw_directory, target_directory):
    """
    Processes and builds the appropriate output for a given raw story.
    """
    print("Processing {}...".format(story_name), end='')
    raw_story = os.path.join(raw_directory, story_name)
    target_story = os.path.join(target_directory, story_name)

    with codecs.open(raw_story, encoding="utf-8") as f:
        story_json = json.load(f)

    cases = story_json["cases"]
    seqno = 0
    output_data = {
        "cases": [],
        "description": (
            "Encoded headers produced by the Python HPACK library, "
            "version {}".format(hpack.__version__)
        ),
    }

    encoder = hpack.Encoder()

    for case in cases:
        # First we need to resize the header table.
        table_size = case.get("header_table_size")
        if table_size is not None:
            encoder.header_table_size = table_size

        headers = headers_from_story(case["headers"])
        result = binascii.hexlify(encoder.encode(headers)).decode('ascii')

        # Provide the result
        output_case = {
            "seqno": seqno,
            "wire": result,
            "headers": case["headers"],
        }
        if table_size is not None:
            output_case["header_table_size"] = table_size

        output_data["cases"].append(output_case)
        seqno += 1

    with codecs.open(target_story, mode="wb", encoding="utf-8") as f:
        json.dump(output_data, f, indent=2, sort_keys=True)

    print("complete.")


def main():
    test_repository = sys.argv[1]
    if not os.path.isdir(test_repository):
        print("{} is not a valid repository path".format(test_repository))
        sys.exit(1)

    # Work out what our input files are.
    raw_story_directory = os.path.join(test_repository, "raw-data")
    raw_story_files = os.listdir(raw_story_directory)

    # Get a place ready to write our output files.
    target_directory = os.path.join(test_repository, "python-hpack")
    os.mkdir(target_directory)

    for story in raw_story_files:
        process_story(story, raw_story_directory, target_directory)


if __name__ == '__main__':
    main()
