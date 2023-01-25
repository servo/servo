# This script can convert IdnaTestV2.txt to JSON, accounting for the requirements in the
# URL Standard.
#
# The goal is to eventually remove --exclude-std3 and --exclude-bidi. For that we need solutions to
# these issues:
#
# * https://github.com/whatwg/url/issues/341
# * https://github.com/whatwg/url/issues/543
# * https://github.com/whatwg/url/issues/733
# * https://github.com/whatwg/url/issues/744
#
# Removal of --exclude-ipv4-like is a stretch goal also dependent upon those issues.

import argparse
import json
import os
import re
import requests

def get_IdnaTestV2_lines():
    IdnaTestV2 = os.path.join(os.path.dirname(__file__), "IdnaTestV2.txt")
    if not os.path.exists(IdnaTestV2):
        # Download IdnaTestV2.txt if it doesn't exist yet
        open(IdnaTestV2, "w").write(requests.get("https://unicode.org/Public/idna/latest/IdnaTestV2.txt").text)
    return open(IdnaTestV2, "r").readlines()

def remove_escapes(input):
    return json.loads("\"" + input + "\"")

def ends_in_a_number(input):
    # This method is not robust. It uses https://www.unicode.org/reports/tr46/#Notation but there
    # are likely other ways to end up with a dot, e.g., through decomposition or percent-decoding.
    # It also does not entirely match https://url.spec.whatwg.org/#ends-in-a-number-checker. It
    # appears to suffice for the tests in question though.
    parts = re.split(r"\u002E|\uFF0E|\u3002|\uFF61", input)
    if not parts:
        return False
    if parts[-1] == "":
        if len(parts) == 1:
            return False
        parts.pop()
    return parts[-1].isascii() and parts[-1].isdigit()

def contains_bidi_status(statuses):
    for status in statuses:
        if status in ["B1", "B2", "B3", "B4", "B5", "B6"]:
            return True
    return False

def parse(lines, exclude_ipv4_like, exclude_std3, exclude_bidi):
    # Main quest.
    output = ["THIS IS A GENERATED FILE. PLEASE DO NOT MODIFY DIRECTLY. See ../tools/IdnaTestV2-parser.py instead."]
    output.append(f"--exclude-ipv4-like: {exclude_ipv4_like}; --exclude-std3: {exclude_std3}; --exclude_bidi: {exclude_bidi}")

    # Side quest.
    unique_statuses = []

    for line in lines:
        # Remove newlines
        line = line.rstrip()

        # Remove lines that are comments or empty
        if line.startswith("#") or line == "":
            continue

        # Remove escapes (doesn't handle \x{XXXX} but those do not appear in the source)
        line = remove_escapes(line)

        # Normalize columns
        #
        # Since we are only interested in ToASCII and enforce Transitional_Processing=false we care
        # about the following columns:
        #
        # * Column 1 (source)
        # * Column 4 (toAsciiN)
        # * Column 5 (toAsciiNStatus)
        #
        # We also store Column 2 (toUnicode) to help with UseSTD3ASCIIRules exclusion.
        columns = [column.strip() for column in line.split(";")]

        # Column 1 (source) and Column 2 (toUnicode; if empty, Column 1 (source))
        source = columns[0]
        to_unicode = columns[1]
        if to_unicode == "":
            to_unicode = source

        # Immediately exclude IPv4-like tests when desired. While we could force all their
        # expectations to be failure instead, it's not clear we need that many additional tests that
        # were actually trying to test something else.
        if exclude_ipv4_like:
            if ends_in_a_number(source):
                continue

        if exclude_std3:
            if re.search(r"\u2260|\u226E|\u226F|\<|\>|\$|,", to_unicode):
                continue

        # Column 4 (toAsciiN; if empty, use Column 2 (toUnicode))
        to_ascii = columns[3]
        if to_ascii == "":
            to_ascii = to_unicode

        # Column 5 (toAsciiNStatus; if empty, use Column 3 (toUnicodeStatus))
        temp_statuses = columns[4]
        if temp_statuses == "":
            temp_statuses = columns[2]

        statuses = []
        if temp_statuses != "":
            assert temp_statuses.startswith("[")
            statuses = [status.strip() for status in temp_statuses[1:-1].split(",")]

        # Side quest time.
        for status in statuses:
            if status not in unique_statuses:
                unique_statuses.append(status)

        # The URL Standard has
        #
        # * UseSTD3ASCIIRules=false; however there are no tests marked U1 (some should be though)
        # * CheckHyphens=false; thus ignore V2, V3?
        # * VerifyDnsLength=false; thus ignore A4_1 and A4_2
        ignored_statuses = []
        for status in statuses:
            if status in ["A4_1", "A4_2", "U1", "V2", "V3"]:
                ignored_statuses.append(status)
        for status in ignored_statuses:
            statuses.remove(status)

        if exclude_bidi and contains_bidi_status(statuses):
            continue

        if len(statuses) > 0:
            to_ascii = None

        test = { "input": source, "output": to_ascii }
        comment = ""
        for status in statuses:
            comment += status + "; "
        for status in ignored_statuses:
            comment += status + " (ignored); "
        if comment != "":
            test["comment"] = comment.strip()[:-1]
        output.append(test)

    unique_statuses.sort()
    return { "tests": output, "unique_statuses": unique_statuses }

def to_json(data):
    handle = open(os.path.join(os.path.dirname(__file__), "../resources/IdnaTestV2.json"), "w")
    handle.write(json.dumps(data, sort_keys=True, allow_nan=False, indent=2, separators=(',', ': ')))
    handle.write("\n")
    handle.close()

def main():
    parser = argparse.ArgumentParser(epilog="Thanks for caring about IDNA!")
    parser.add_argument("--generate", action="store_true", help="Generate the JSON resource.")
    parser.add_argument("--exclude-ipv4-like", action="store_true", help="Exclude inputs that end with an ASCII digit label. (Not robust, but works for current input.)")
    parser.add_argument("--exclude-std3", action="store_true", help="Exclude tests impacted by UseSTD3ASCIIRules. (Not robust, but works for current input.)")
    parser.add_argument("--exclude-bidi", action="store_true", help="Exclude tests impacted by CheckBidi.")
    parser.add_argument("--statuses", action="store_true", help="Print the unique statuses in IdnaTestV2.txt.")
    args = parser.parse_args()

    if args.generate or args.statuses:
        output = parse(get_IdnaTestV2_lines(), args.exclude_ipv4_like, args.exclude_std3, args.exclude_bidi)
        if args.statuses:
            print(output["unique_statuses"])
        else:
            assert args.generate
            to_json(output["tests"])
    else:
        parser.print_usage()

main()
