#!/usr/bin/env python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

# This is a script designed to easily debug devtools messages
# It takes the content of a pcap wireshark capture (or creates a new
# one when using --scan) and pretty prints the JSON payloads.
#
# Wireshark (more specifically its cli tool tshark) needs to be installed
# for this script to work. Go to https://tshark.dev/setup/install for a
# comprehensive guide on how to install it. In short:
#
# Linux (Debian based):       apt install tshark
# Linux (Arch based):         pacman -Sy wireshark-cli
# MacOS (With homebrew):      brew install --cask wireshark
# Windows (With chocolatey):  choco install wireshark
#
# To use it, launch either Servo or a Firefox debugging instance in
# devtools mode:
#
# Servo: ./mach run --devtools=1234
# Firefox: firefox --new-instance --start-debugger-server 1234 --profile PROFILE
#
# Then run this tool in capture mode and specify the same port as before:
#
# ./devtools_parser.py --scan cap.pcap --port 1234
#
# Finally, open another instance of Firefox and go to about:debugging
# and connect to localhost:1234. Messages should start popping up. The
# scan can be finished by pressing Ctrl+C. Then, all of the messages will
# show up.
#
# You can also review the results of a saved scan, and filter by words
# or by message range:
#
# ./devtools_parser.py --use cap.pcap --port 1234 --filter watcher --range 10:30

import json
import re
import signal
from argparse import ArgumentParser
from subprocess import Popen, PIPE
try:
    from termcolor import colored
except ImportError:
    def colored(text, *args, **kwargs):
        return text

fields = ["frame.time", "tcp.srcport", "tcp.payload"]
pattern = r"(\d+:){"


# Use tshark to capture network traffic and save the result in a
# format that this tool can process later
def record_data(file, port):
    # Create tshark command
    cmd = [
        "tshark",
        "-T", "fields",
        "-i", "lo",
        "-d", f"tcp.port=={port},http",
        "-w", file,
    ] + [e for f in fields for e in ("-e", f)]
    process = Popen(cmd, stdout=PIPE)

    # Stop the analysis when using Ctrl+C
    def signal_handler(sig, frame):
        process.kill()

    signal.signal(signal.SIGINT, signal_handler)
    signal.pause()

    # Get the output
    out, err = process.communicate()
    out = out.decode("utf-8")

    return out


# Read a pcap data file from tshark (or wireshark) and extract
# the necessary output fields
def read_data(file):
    # Create tshark command
    cmd = [
        "tshark",
        "-T", "fields",
        "-r", file,
    ] + [e for f in fields for e in ("-e", f)]
    process = Popen(cmd, stdout=PIPE)

    # Get the output
    out, err = process.communicate()
    out = out.decode("utf-8")

    return out


# Transform the raw output of wireshark into a more manageable one
def process_data(out, port):
    out = [line.split("\t") for line in out.split("\n")]

    # Process fields
    data = []
    for line in out:
        if len(line) != 3:
            continue

        curr_time, curr_port, curr_data = line

        # Time
        curr_time = curr_time.split(" ")[-2].split(".")[0]
        # Port
        curr_port = "Servo" if curr_port == port else "Firefox"
        # Data
        if not curr_data:
            continue
        try:
            dec = bytearray.fromhex(curr_data).decode()
        except UnicodeError:
            continue

        indices = [m.span() for m in re.finditer(pattern, dec)]
        indices.append((len(dec), -1))
        prev = 0

        for min, max in indices:
            if min > 0:
                text = dec[prev - 1:min]
                data.append([curr_time, curr_port, len(data), text])
            prev = max

    return data


# Pretty prints the json message
def parse_message(msg):
    time, sender, i, data = msg
    from_servo = sender == "Servo"

    colored_sender = colored(sender, 'black', 'on_yellow' if from_servo else 'on_magenta', attrs=['bold'])
    print(f"\n{colored_sender} - {colored(i, 'blue')} - {colored(time, 'dark_grey')}")

    try:
        content = json.loads(data)
        if from_servo and "from" in content:
            print(colored(f"Actor: {content['from']}", 'yellow'))
        print(json.dumps(content, sort_keys=True, indent=4))
    except json.JSONDecodeError:
        print(f"Warning: Couldn't decode json\n{data}")

    print()


if __name__ == "__main__":
    # Program arguments
    parser = ArgumentParser()
    parser.add_argument("-p", "--port", default="1234", help="the port where the devtools client is running")
    parser.add_argument("-f", "--filter", help="search for the string on the messages")
    parser.add_argument("-r", "--range", help="only parse messages from n to m, with the form of n:m")

    actions = parser.add_mutually_exclusive_group(required=True)
    actions.add_argument("-s", "--scan", help="scan and save the output to a file")
    actions.add_argument("-u", "--use", help="use the scan from a file")

    args = parser.parse_args()

    # Get the scan data
    if args.scan:
        data = record_data(args.scan, args.port)
    else:
        with open(args.use, "r") as f:
            data = read_data(args.use)

    data = process_data(data, args.port)

    # Set the range of messages to show
    min, max = 0, -2
    if args.range and len(args.range.split(":")) == 2:
        min, max = args.range.split(":")

    for msg in data[int(min):int(max) + 1]:
        # Filter the messages if specified
        if not args.filter or args.filter.lower() in msg[3].lower():
            parse_message(msg)
