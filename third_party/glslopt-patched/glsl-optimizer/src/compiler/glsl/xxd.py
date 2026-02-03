# encoding=utf-8
# Copyright © 2018 Intel Corporation
#
# Permission is hereby granted, free of charge, to any person obtaining a
# copy of this software and associated documentation files (the "Software"),
# to deal in the Software without restriction, including without limitation
# the rights to use, copy, modify, merge, publish, distribute, sublicense,
# and/or sell copies of the Software, and to permit persons to whom the
# Software is furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice (including the next
# paragraph) shall be included in all copies or substantial portions of the
# Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL
# THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
# FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
# IN THE SOFTWARE.

# Converts a file to a C/C++ #include containing a string

from __future__ import unicode_literals
import argparse
import io
import string
import sys


def get_args():
    parser = argparse.ArgumentParser()
    parser.add_argument('input', help="Name of input file")
    parser.add_argument('output', help="Name of output file")
    parser.add_argument("-n", "--name",
                        help="Name of C variable")
    args = parser.parse_args()
    return args


def filename_to_C_identifier(n):
    if n[0] != '_' and not n[0].isalpha():
        n = "_" + n[1:]

    return "".join([c if c.isalnum() or c == "_" else "_" for c in n])


def emit_byte(f, b):
    if ord(b) == ord('\n'):
        f.write(b"\\n\"\n    \"")
        return
    elif ord(b) == ord('\r'):
        f.write(b"\\r\"\n    \"")
        return
    elif ord(b) == ord('\t'):
        f.write(b"\\t")
        return
    elif ord(b) == ord('"'):
        f.write(b"\\\"")
        return
    elif ord(b) == ord('\\'):
        f.write(b"\\\\")
        return

    if ord(b) >= ord(' ') and ord(b) <= ord('~'):
        f.write(b)
    else:
        hi = ord(b) >> 4
        lo = ord(b) & 0x0f
        f.write("\\x{:x}{:x}".format(hi, lo).encode('utf-8'))


def process_file(args):
    with io.open(args.input, "rb") as infile:
        try:
            with io.open(args.output, "wb") as outfile:
                # If a name was not specified on the command line, pick one based on the
                # name of the input file.  If no input filename was specified, use
                # from_stdin.
                if args.name is not None:
                    name = args.name
                else:
                    name = filename_to_C_identifier(args.input)

                outfile.write("static const char {}[] =\n \"".format(name).encode('utf-8'))

                while True:
                    byte = infile.read(1)
                    if byte == b"":
                        break

                    emit_byte(outfile, byte)

                outfile.write(b"\"\n    ;\n")
        except Exception:
            # In the event that anything goes wrong, delete the output file,
            # then re-raise the exception. Deleteing the output file should
            # ensure that the build system doesn't try to use the stale,
            # half-generated file.
            os.unlink(args.output)
            raise


def main():
    args = get_args()
    process_file(args)


if __name__ == "__main__":
    main()
