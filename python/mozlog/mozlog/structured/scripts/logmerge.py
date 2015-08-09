from __future__ import print_function
import argparse
import json
import os
import sys
from threading import current_thread
import time
from mozlog.structured.reader import read


def dump_entry(entry, output):
    json.dump(entry, output)
    output.write("\n")


def fill_process_info(event):
    event["time"] = int(round(time.time() * 1000))
    event["thread"] = current_thread().name
    event["pid"] = os.getpid()
    return event


def process_until(reader, output, action):
    for entry in reader:
        if entry['action'] == action:
            return entry
        dump_entry(entry, output)


def process_until_suite_start(reader, output):
    return process_until(reader, output, "suite_start")


def process_until_suite_end(reader, output):
    return process_until(reader, output, "suite_end")


def validate_start_events(events):
    for start in events:
        if not start['run_info'] == events[0]['run_info']:
            print("Error: different run_info entries", file=sys.stderr)
            sys.exit(1)


def merge_start_events(events):
    for start in events[1:]:
        events[0]["tests"].extend(start["tests"])
    return events[0]


def get_parser(add_help=True):
    parser = argparse.ArgumentParser("logmerge", description='Merge multiple log files.', add_help=add_help)
    parser.add_argument('-o', dest='output', help='output file, defaults to stdout')
    parser.add_argument('files', metavar='File', type=str, nargs='+', help='file to be merged')
    return parser


def main(**kwargs):
    if kwargs["output"] is None:
        output = sys.stdout
    else:
        output = open(kwargs["output"], "w")
    readers = [read(open(filename, 'r')) for filename in kwargs["files"]]
    start_events = [process_until_suite_start(reader, output) for reader in readers]
    validate_start_events(start_events)
    merged_start_event = merge_start_events(start_events)
    dump_entry(fill_process_info(merged_start_event), output)

    end_events = [process_until_suite_end(reader, output) for reader in readers]
    dump_entry(fill_process_info(end_events[0]), output)

    for reader in readers:
        for entry in reader:
            dump_entry(entry, output)



if __name__ == "__main__":
    parser = get_parser()
    args = parser.parse_args()
    kwargs = vars(args)
    main(**kwargs)