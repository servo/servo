#!/usr/bin/env python3

# Copyright 2024 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import argparse
import json
import os


def size(args):
    size = os.path.getsize(args.binary)
    print(size)
    with open(args.bmf_output, 'w', encoding='utf-8') as f:
        json.dump({
            args.variant: {
                'file-size': {
                    'value': float(size),
                }
            }
        }, f, indent=4)


def merge(args):
    output: dict[str, object] = dict()
    for input_file in args.inputs:
        with open(input_file, 'r', encoding='utf-8') as f:
            data = json.load(f)
            diff = set(data) & set(output)
            if diff:
                print("Duplicated keys:", diff)
            output = data | output

    with open(args.bmf_output, 'w', encoding='utf-8') as f:
        json.dump(output, f, indent=4)


parser = argparse.ArgumentParser("Helper commands for bencher")

subparser = parser.add_subparsers()
size_parser = subparser.add_parser("filesize", help="Returns BMF for filesize")
size_parser.add_argument("binary", help="Servo binary file")
size_parser.add_argument("variant", help="variant of the binary")
size_parser.add_argument("--bmf-output", help="BMF JSON output file", default=None)
size_parser.set_defaults(func=size)

merge_parser = subparser.add_parser("merge", help="Merges BMF JSONs")
merge_parser.add_argument("--bmf-output", help="BMF JSON output file")
merge_parser.add_argument("inputs", help="BMF JSON files to merge", nargs="+")
merge_parser.set_defaults(func=merge)

args = parser.parse_args()
args.func(args)
