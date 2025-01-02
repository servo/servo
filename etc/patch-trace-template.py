#!/usr/bin/env python

# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import re
import subprocess
import sys
from xml.etree import ElementTree
from xml.etree.ElementTree import Element

if len(sys.argv) < 3:
    sys.stderr.write("""usage: patch-trace-template.py components/profile_traits/time.rs path/to/my.tracetemplate

`time.rs` is usually located in `components/profile_traits/time.rs`.
Trace templates are typically located in `~/Library/Application Support/Instruments/Templates`.
The supplied trace template must contain the "Points of Interest" instrument.
Output is written to standard output and should typically be piped to a new `.tracetemplate` file.

Example:
    patch-trace-template.py \\
    components/profile_traits/time.rs \\
        ~/Library/Application\\ Support/Instruments/Templates/MyTemplate.tracetemplate > \\
        ~/Library/Application\\ Support/Instruments/Templates/MyServoTemplate.tracetemplate
""")
    sys.exit(0)

rust_source = open(sys.argv[1], 'r')
lines = iter(rust_source)
for line in lines:
    if line.lstrip().startswith("pub enum ProfilerCategory"):
        break

code_pairs = []
regex = re.compile(r"\s*(\w+)\s*=\s*([0-9]+|0x[a-fA-F0-9]+),?\s*$")
for line in lines:
    if line.lstrip().startswith("}"):
        break

    match = regex.match(line)
    if match is None:
        continue
    code_pairs.append((match.group(2), match.group(1)))

xml = subprocess.check_output(["plutil", "-convert", "xml1", "-o", "-", sys.argv[2]])
plist = ElementTree.ElementTree(ElementTree.fromstring(xml))

elems = iter(plist.findall("./dict/*"))
for elem in elems:
    if elem.tag != 'key' or elem.text != '$objects':
        continue
    array = elems.next()
    break

elems = iter(array.findall("./*"))
for elem in elems:
    if elem.tag != 'string' or elem.text != 'kdebugIntervalRule':
        continue
    dictionary = elems.next()
    break

elems = iter(dictionary.findall("./*"))
for elem in elems:
    if elem.tag != 'key' or elem.text != 'NS.objects':
        continue
    objects_array = elems.next()
    break

child_count = sum(1 for _ in iter(array.findall("./*")))

for code_pair in code_pairs:
    number_index = child_count
    integer = Element('integer')
    integer.text = str(int(code_pair[0], 0))
    array.append(integer)
    child_count += 1

    string_index = child_count
    string = Element('string')
    string.text = code_pair[1]
    array.append(string)
    child_count += 1

    dictionary = Element('dict')
    key = Element('key')
    key.text = "CF$UID"
    dictionary.append(key)
    integer = Element('integer')
    integer.text = str(number_index)
    dictionary.append(integer)
    objects_array.append(dictionary)

    dictionary = Element('dict')
    key = Element('key')
    key.text = "CF$UID"
    dictionary.append(key)
    integer = Element('integer')
    integer.text = str(string_index)
    dictionary.append(integer)
    objects_array.append(dictionary)

plist.write(sys.stdout, encoding='utf-8', xml_declaration=True)
