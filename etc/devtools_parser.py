#!/usr/bin/env python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

# This is a script designed to easily debug devtools messages
# It takes the content of a pcap wireshark capture (or creates a new
# one when using `-w`) and prints the JSON payloads.
#
# Wireshark (more specifically its cli tool tshark) needs to be installed
# for this script to work. Go to https://tshark.dev/setup/install for a
# comprehensive guide on how to install it. In short:
#
# Linux (Debian based):       apt install tshark
# Linux (Arch based):         pacman -Sy wireshark-cli
# MacOS (With homebrew):      brew install --cask wireshark
# Windows (With winget):  winget install --exact WiresharkFoundation.Wireshark
#
# To use it, launch Servo or Firefox in devtools mode:
#
# Servo: ./mach run --devtools 6080
# Firefox: firefox --new-instance --start-debugger-server 6080 --profile PROFILE
#
# Then run this tool in capture mode, specifying the same port as before:
#
# ./devtools_parser.py -w capture.pcap -p 6080
#
# Finally, open another instance of Firefox, go to about:debugging and connect
# to localhost:6080. Messages should start popping up. The scan can be finished
# by pressing Ctrl+C. After that the messages will be printed.
#
# To review the results of the scan use the `-r` flag. It is possible to output
# newline-delimited JSON for further processing with other tools using the
# `--json` flag.
#
# ./devtools_parser.py -r capture.pcap --json > capture.json

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


def tshark(args, wait=False):
    """Run a tshark command with the specified arguments.
    Some custom arguments are always prepended to set the stdout format: date, port, hex-encoded data.
    If `wait` is True, the main process will pause until SIGINT is triggered. This action will stop the analysis and continue execution."""

    cmd = ["tshark", "-T", "fields", "-e", "frame.time", "-e", "tcp.srcport", "-e", "tcp.payload"] + args
    process = Popen(cmd, stdout=PIPE, encoding="utf-8")

    if wait:
        signal.signal(signal.SIGINT, lambda _signal, _frame: process.send_signal(signal.SIGINT))
        signal.pause()

    return process.communicate()[0]


def process_data(input):
    """Transform the raw output of tshark stdout into a manageable list."""

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
                records.append([time, message.decode()])
            except UnicodeError as e:
                print(f"[WARNING] Failed to decode message as UTF-8: {e}")
                continue

    # Return enumerated records.
    # `records` = [[date, message text]], e.g.
    # `["2025-11-04T16:01:38.013100950+0100", "{...}"]`
    # `return` = [[date, message text, index]], e.g.
    # `["2025-11-04T16:01:38.013100950+0100", "{...}", 0]`
    return [(*line, i) for i, line in enumerate(records)]


def parse_message(msg, json_output=False):
    """Pretty print the JSON message, actor and timestamp.
    If `json_output` is True, output the JSON message in one line instead."""

    time, data, i = msg

    try:
        content = json.loads(data)
    except json.JSONDecodeError:
        print(f"Warning: Couldn't decode json\n{data}")
        return

    if json_output:
        # Place from and to at the start so that it is easier to see which actor is involved
        sorted_content = dict(
            sorted(content.items(), key=lambda k: f"_{k[0]}" if k[0] == "from" or k[0] == "to" else k[0])
        )
        print(json.dumps(sorted_content))
        return

    is_server = "from" in content
    colored_sender = (
        colored("Server", "black", "on_yellow") if is_server else colored("Client", "on_magenta", attrs=["bold"])
    )
    pretty_json = json.dumps(content, sort_keys=True, indent=4)

    print(f"""
{colored_sender} - {colored(i, "blue")} - {colored(time, "dark_grey")}
{pretty_json}
""")


if __name__ == "__main__":
    # Program arguments
    parser = ArgumentParser()
    parser.add_argument("-p", "--port", default="6080", help="the port where the devtools client is running")
    parser.add_argument("--json", action="store_true", help="output in newline-delimited JSON (NDJSON)")

    actions = parser.add_mutually_exclusive_group(required=True)
    actions.add_argument(
        "-w", "--write-file", help="capture messages on the specified port and write the output to a .pcap file"
    )
    actions.add_argument("-r", "--read-file", help="parse the captured messages from a .pcap file")

    args = parser.parse_args()

    # Run tshark, either to start a capture or to read an already existing pcap file
    if args.write_file:
        capture_args = ["-i", "lo", "-f", f"tcp port {args.port}", "-w", args.write_file]
        data = tshark(capture_args, wait=True)
    else:
        read_args = ["-r", args.read_file]
        data = tshark(read_args)

    data = process_data(data)

    for msg in data:
        parse_message(msg, json_output=args.json)
