#!/usr/bin/env python3

# Copyright 2018 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import json
import os
import sys
import tempfile
import webbrowser


def extract_memory_reports(lines):
    in_report = False
    report_lines = []
    times = []
    for line in lines:
        if line.startswith('Begin memory reports'):
            in_report = True
            report_lines += [[]]
            times += [line.strip().split()[-1]]
        elif line == 'End memory reports\n':
            in_report = False
        elif in_report:
            if line.startswith('|'):
                report_lines[-1].append(line.strip())
    return (report_lines, times)


def parse_memory_report(lines):
    reports = {}
    parents = []
    last_separator_index = None
    for line in lines:
        assert (line[0] == '|')
        line = line[1:]
        if not line:
            continue
        separator_index = line.index('--')
        if last_separator_index and separator_index <= last_separator_index:
            while parents and parents[-1][1] >= separator_index:
                parents.pop()

        amount, unit, _, name = line.split()

        dest_report = reports
        for (parent, index) in parents:
            dest_report = dest_report[parent]['children']
        dest_report[name] = {
            'amount': amount,
            'unit': unit,
            'children': {}
        }

        parents += [(name, separator_index)]
        last_separator_index = separator_index
    return reports


def transform_report_for_test(report):
    transformed = {}
    remaining = list(report.items())
    while remaining:
        (name, value) = remaining.pop()
        transformed[name] = '%s %s' % (value['amount'], value['unit'])
        remaining += map(lambda k_v: (name + '/' + k_v[0], k_v[1]), list(value['children'].items()))
    return transformed


def test_extract_memory_reports():
    input = ["Begin memory reports",
             "|",
             "  154.56 MiB -- explicit\n",
             "|     107.88 MiB -- system-heap-unclassified\n",
             "End memory reports\n"]
    expected = ([['|', '|     107.88 MiB -- system-heap-unclassified']], ['reports'])
    assert (extract_memory_reports(input) == expected)
    return 0


def test():
    input = '''|
|   23.89 MiB -- explicit
|      21.35 MiB -- jemalloc-heap-unclassified
|       2.54 MiB -- url(https://servo.org/)
|          2.16 MiB -- js
|             1.00 MiB -- gc-heap
|                0.77 MiB -- decommitted
|             1.00 MiB -- non-heap
|          0.27 MiB -- layout-thread
|             0.27 MiB -- stylist
|          0.12 MiB -- dom-tree
|
|   25.18 MiB -- jemalloc-heap-active'''

    expected = {
        'explicit': '23.89 MiB',
        'explicit/jemalloc-heap-unclassified': '21.35 MiB',
        'explicit/url(https://servo.org/)': '2.54 MiB',
        'explicit/url(https://servo.org/)/js': '2.16 MiB',
        'explicit/url(https://servo.org/)/js/gc-heap': '1.00 MiB',
        'explicit/url(https://servo.org/)/js/gc-heap/decommitted': '0.77 MiB',
        'explicit/url(https://servo.org/)/js/non-heap': '1.00 MiB',
        'explicit/url(https://servo.org/)/layout-thread': '0.27 MiB',
        'explicit/url(https://servo.org/)/layout-thread/stylist': '0.27 MiB',
        'explicit/url(https://servo.org/)/dom-tree': '0.12 MiB',
        'jemalloc-heap-active': '25.18 MiB',
    }
    report = parse_memory_report(input.split('\n'))
    transformed = transform_report_for_test(report)
    assert (sorted(transformed.keys()) == sorted(expected.keys()))
    for k, v in transformed.items():
        assert (v == expected[k])
    test_extract_memory_reports()
    return 0


def usage():
    print('%s --test - run automated tests' % sys.argv[0])
    print('%s file - extract all memory reports that are present in file' % sys.argv[0])
    return 1


if __name__ == "__main__":
    if len(sys.argv) == 1:
        sys.exit(usage())

    if sys.argv[1] == '--test':
        sys.exit(test())

    with open(sys.argv[1]) as f:
        lines = f.readlines()
    (reports, times) = extract_memory_reports(lines)
    json_reports = []
    for (report_lines, seconds) in zip(reports, times):
        report = parse_memory_report(report_lines)
        json_reports += [{'seconds': seconds, 'report': report}]
    with tempfile.NamedTemporaryFile(delete=False) as output:
        thisdir = os.path.dirname(os.path.abspath(__file__))
        with open(os.path.join(thisdir, 'memory_chart.html')) as template:
            content = template.read()
            output.write(content.replace('[/* json data */]', json.dumps(json_reports)))
            webbrowser.open_new_tab('file://' + output.name)
