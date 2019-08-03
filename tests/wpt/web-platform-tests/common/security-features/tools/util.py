from __future__ import print_function

import os, sys, json, re

script_directory = os.path.dirname(os.path.abspath(__file__))
template_directory = os.path.abspath(
    os.path.join(script_directory, 'template'))
test_root_directory = os.path.abspath(
    os.path.join(script_directory, '..', '..', '..'))


def get_template(basename):
    with open(os.path.join(template_directory, basename), "r") as f:
        return f.read()


def write_file(filename, contents):
    with open(filename, "w") as f:
        f.write(contents)


def read_nth_line(fp, line_number):
    fp.seek(0)
    for i, line in enumerate(fp):
        if (i + 1) == line_number:
            return line


def load_spec_json(path_to_spec):
    re_error_location = re.compile('line ([0-9]+) column ([0-9]+)')
    with open(path_to_spec, "r") as f:
        try:
            return json.load(f)
        except ValueError as ex:
            print(ex.message)
            match = re_error_location.search(ex.message)
            if match:
                line_number, column = int(match.group(1)), int(match.group(2))
                print(read_nth_line(f, line_number).rstrip())
                print(" " * (column - 1) + "^")
            sys.exit(1)


class PolicyDelivery(object):
    '''
    See `@typedef PolicyDelivery` comments in `resources/common.js`.
    '''

    def __init__(self, delivery_type, key, value):
        self.delivery_type = delivery_type
        self.key = key
        self.value = value
