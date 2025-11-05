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
import signal
import sys
from argparse import ArgumentParser
from subprocess import Popen, PIPE

try:
    from termcolor import colored
except ImportError:

    def colored(text, *args, **kwargs):
        return text


# Run a tshark command
# If wait is true, the main process will stop until SIGINT is passed, stopping the analysis
def tshark(args, wait=False):
    cmd = ["tshark", "-T", "fields", "-e", "frame.time", "-e", "tcp.srcport", "-e", "tcp.payload"] + args
    process = Popen(cmd, stdout=PIPE, encoding="utf-8")

    if wait:
        signal.signal(signal.SIGINT, lambda _signal, _frame: process.send_signal(signal.SIGINT))
        signal.pause()

    return process.communicate()[0]


# Transform the raw output of wireshark into a more manageable one
def process_data(input, servo_port):
    # Split the input into lines.
    # `input` = newline-terminated lines of tab-delimited tshark(1) output
    lines = [line.split("\t") for line in input.split("\n")]

    # Remove empty lines and empty messages, and decode hex to bytes.
    # `lines` = [[date, port, hex-encoded data]], e.g.
    # `["2025-11-04T16:01:38.013100950+0100", "6080", "3133"]`
    # `["2025-11-04T16:01:38.013100950+0100", "6080", "393a"]`
    # `["2025-11-04T16:01:38.013100950+0100", "6080", "7b..."]`
    messages = []
    for line in lines:
        if len(line) != 3:
            continue
        time, port, data = line
        if len(data) == 0:
            continue
        elif len(data) % 2 == 1:
            print(f"[WARNING] Extra byte in hex-encoded data: {data[-1]}", file=sys.stderr)
            data = data[:-1]
        if len(messages) > 0 and messages[-1][1] == port:
            messages[-1][2] += bytearray.fromhex(data)
        else:
            messages.append([time, port, bytearray.fromhex(data)])

    # Split and merge consecutive messages with the same port, to yield exactly one record per message.
    # Message records are of the form `length:{...}`, where `length` is an integer in ASCII decimal.
    # Incomplete messages are deferred until they are complete.
    # `sends` = [[date, port, record data]], e.g.
    # `["2025-11-04T16:01:38.013100950+0100", "6080", b"13"]`
    # `["2025-11-04T16:01:38.013100950+0100", "6080", b"9:"]`
    # `["2025-11-04T16:01:38.013100950+0100", "6080", b"{..."]`
    # `["2025-11-04T16:01:38.013100950+0100", "6080", b"...}"]`
    records = []
    scunge = {}  # Map from port to incomplete message data
    for time, port, rest in messages:
        rest = scunge.pop(port, b"") + rest
        while rest != b"":
            try:
                length, new_rest = rest.split(b":", 1)  # Can raise ValueError
                length = int(length)
                if len(new_rest) < length:
                    raise ValueError("Incomplete message (for now)")
                # If we found a `length:` prefix and we have enough data to satisfy it,
                # cut off the prefix so `rest` is just `{...}length:{...}length:{...}`.
                rest = new_rest
            except ValueError:
                print(f"[WARNING] Incomplete message detected (will try to reassemble): {repr(rest)}", file=sys.stderr)
                scunge[port] = rest
                # Wait for more data from later sends, potentially after sends with the other port.
                break
            # Cut off the message so `rest` is just `length:{...}length:{...}`.
            message = rest[:length]
            rest = rest[length:]
            try:
                records.append([time, port, message.decode()])
            except UnicodeError as e:
                print(f"[WARNING] Failed to decode message as UTF-8: {e}")
                continue

    # Process message records.
    # `records` = [[date, port, message text]], e.g.
    # `["2025-11-04T16:01:38.013100950+0100", "6080", "{...}"]`
    result = []
    for line in records:
        time, port, text = line
        # Port
        port = "Servo" if port == servo_port else "Firefox"
        # Data
        result.append([time, port, len(result), text])

    # `result` = [[date, endpoint, index, message text]], e.g.
    # `["2025-11-04T16:01:38.013100950+0100", "Servo", 0, "{...}"]`
    return result


# Pretty prints the json message
def parse_message(msg, *, json_output=False):
    time, sender, i, data = msg
    from_servo = sender == "Servo"

    colored_sender = colored(sender, "black", "on_yellow" if from_servo else "on_magenta", attrs=["bold"])
    if not json_output:
        print(f"\n{colored_sender} - {colored(i, 'blue')} - {colored(time, 'dark_grey')}")

    try:
        content = json.loads(data)
        if json_output:
            if "to" in content:
                # This is a request
                print(json.dumps({"_to": content["to"], "message": content}, sort_keys=True))
            elif "from" in content:
                # This is a response
                print(json.dumps({"_from": content["from"], "message": content}, sort_keys=True))
            else:
                assert False, "Message is neither a request nor a response"
        else:
            if from_servo and "from" in content:
                print(colored(f"Actor: {content['from']}", "yellow"))
            print(json.dumps(content, sort_keys=True, indent=4))
    except json.JSONDecodeError:
        print(f"Warning: Couldn't decode json\n{data}")

    if not json_output:
        print()


if __name__ == "__main__":
    # Program arguments
    parser = ArgumentParser()
    parser.add_argument("-p", "--port", default="1234", help="the port where the devtools client is running")
    parser.add_argument("-f", "--filter", help="search for the string on the messages")
    parser.add_argument("-r", "--range", help="only parse messages from n to m, with the form of n:m")
    parser.add_argument("--json", action="store_true", help="output in newline-delimited JSON (NDJSON)")

    actions = parser.add_mutually_exclusive_group(required=True)
    actions.add_argument("-s", "--scan", help="scan and save the output to a file")
    actions.add_argument("-u", "--use", help="use the scan from a file")

    args = parser.parse_args()

    # Run tshark, either to start a capture or to read an already existing pcap file
    if args.scan:
        capture_args = ["-i", "lo", "-f", f"tcp port {args.port}", "-w", args.scan]
        data = tshark(capture_args, wait=True)
    else:
        read_args = ["-r", args.use]
        data = tshark(read_args)

    data = process_data(data, args.port)

    # Set the range of messages to show
    min, max = 0, -2
    if args.range and len(args.range.split(":")) == 2:
        min, max = args.range.split(":")

    for msg in data[int(min) : int(max) + 1]:
        # Filter the messages if specified
        if not args.filter or args.filter.lower() in msg[3].lower():
            parse_message(msg, json_output=args.json)
