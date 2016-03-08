#!/usr/bin/env python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import sys
import os
from os import path
import time
import datetime
import argparse
import subprocess

TOP_DIR = path.join("..", "..")
GUARD_TIME = 20
SUMMARY_OUTPUT = "summary.txt"


def get_command(layout_thread_count, renderer, page, profile):
    """Get the command to execute.
    """
    return path.join(TOP_DIR, "mach") + " run --android" + \
        " -p %d -o /sdcard/servo/output.png -y %d %s -Z profile-script-events,profile-heartbeats '%s'" % \
        (profile, layout_thread_count, renderer, page)


def git_rev_hash():
    """Get the git revision hash.
    """
    return subprocess.check_output(['git', 'rev-parse', 'HEAD']).rstrip()


def git_rev_hash_short():
    """Get the git revision short hash.
    """
    return subprocess.check_output(['git', 'rev-parse', '--short', 'HEAD']).rstrip()


def execute(base_dir, renderer, page, profile, trial, layout_thread_count):
    """Run a single execution.
    """
    log_dir = path.join(base_dir, "logs_l" + str(layout_thread_count),
                        "trial_" + str(trial))
    if os.path.exists(log_dir):
        print "Log directory already exists: " + log_dir
        sys.exit(1)
    os.makedirs(log_dir)

    # Execute
    cmd = get_command(layout_thread_count, renderer, page, profile)
    print cmd
    os.system(cmd)
    print 'sleep ' + str(GUARD_TIME)
    time.sleep(GUARD_TIME)

    # Write a file that describes this execution
    with open(path.join(log_dir, SUMMARY_OUTPUT), "w") as f:
        f.write("Datetime (UTC): " + datetime.datetime.utcnow().isoformat())
        f.write("\nPlatform: Android")
        f.write("\nGit hash: " + git_rev_hash())
        f.write("\nGit short hash: " + git_rev_hash_short())
        f.write("\nLayout threads: " + str(layout_thread_count))
        f.write("\nTrial: " + str(trial))
        f.write("\nCommand: " + cmd)


def main():
    """For this script to be useful, the following conditions are needed:
    - Build servo for Android in release mode with the "energy-profiling" feature enabled.
    """
    # Default number of layout threads
    layout_threads = 1
    # Default benchmark
    benchmark = "https://www.mozilla.org/"
    # Default renderer
    renderer = ""
    # Default output directory
    output_dir = "heartbeat_logs"
    # Default profile interval
    profile = 60

    # Parsing the input of the script
    parser = argparse.ArgumentParser(description="Characterize Servo timing and energy behavior on Android")
    parser.add_argument("-b", "--benchmark",
                        default=benchmark,
                        help="Gets the benchmark, for example \"-b http://www.example.com\"")
    parser.add_argument("-w", "--webrender",
                        action='store_true',
                        help="Use webrender backend")
    parser.add_argument("-l", "--layout_threads",
                        help="Specify the number of threads for layout, for example \"-l 5\"")
    parser.add_argument("-o", "--output",
                        help="Specify the log output directory, for example \"-o heartbeat_logs\"")
    parser.add_argument("-p", "--profile",
                        default=60,
                        help="Profiler output interval, for example \"-p 60\"")

    args = parser.parse_args()
    if args.benchmark:
        benchmark = args.benchmark
    if args.webrender:
        renderer = "-w"
    if args.layout_threads:
        layout_threads = int(args.layout_threads)
    if args.output:
        output_dir = args.output
    if args.profile:
        profile = args.profile

    if os.path.exists(output_dir):
        print "Output directory already exists: " + output_dir
        sys.exit(1)
    os.makedirs(output_dir)

    execute(output_dir, renderer, benchmark, profile, 1, layout_threads)

if __name__ == "__main__":
    main()
