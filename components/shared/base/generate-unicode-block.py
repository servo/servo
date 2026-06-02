#!/usr/bin/env python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

# The beginning of this script is both valid shell and valid python,
# such that the script starts with the shell and is reexecuted with
# the right python.

import dataclasses
import re
import sys


@dataclasses.dataclass
class UnicodeBlock:
    name: str
    start: str
    end: str


def process_line(line: str) -> UnicodeBlock:
    # Split on either '..' or ';' surrounded by whitespace.
    [start, end, name] = re.split(r"\W*\.\.|;\W*", line, maxsplit=3)
    name = name.strip().replace("-", "").replace(" ", "")
    return UnicodeBlock(name, start.zfill(6), end.zfill(6))


with open(sys.argv[1]) as file:
    lines_to_keep = filter(
        lambda line: line.strip() and not line.startswith("#"),
        file.readlines()
    )
    results = list(map(process_line, lines_to_keep))

print("/* This Source Code Form is subject to the terms of the Mozilla Public")
print(" * License, v. 2.0. If a copy of the MPL was not distributed with this")
print(" * file, You can obtain one at https://mozilla.org/MPL/2.0/. */")
print()
print("// Do not edit:")
print("// Generated via: https://www.unicode.org/Public/UNIDATA/Blocks.txt.")
print("// $ ./generate-unicode-block.py Blocks.txt > unicode_block.rs")
print()
print("#[derive(Clone, Copy, Debug, PartialEq)]")
print("pub enum UnicodeBlock {")
for block in results:
    print(f"    {block.name},")
print("}")
print()
print("pub trait UnicodeBlockMethod {")
print("    fn block(&self) -> Option<UnicodeBlock>;")
print("}")
print()
print("impl UnicodeBlockMethod for char {")
print("    fn block(&self) -> Option<UnicodeBlock> {")
print("        match *self as u32 {")
for block in results:
    print(f"            0x{block.start}..=0x{block.end} => Some(UnicodeBlock::{block.name}),")
print("            _ => None,")
print("        }")
print("    }")
print("}")
