#/usr/bin/env python

import sys, re, subprocess
from sets import Set

symbol_regex = re.compile("D  \*UND\*\t(.*) (.*)$")
allowed_symbols = Set(['unshare', 'malloc_usable_size'])
actual_symbols = Set()

objdump_output = subprocess.check_output([
        'arm-linux-androideabi-objdump',
        '-T',
        'target/arm-linux-androideabi/debug/libservo.so']
    ).split('\n')

for line in objdump_output:
    m = symbol_regex.search(line)
    if m != None:
        actual_symbols.add(m.group(2))

difference = allowed_symbols.difference(actual_symbols)

if len(difference) > 0:
    human_readable_difference = ", ".join(str(s) for s in difference)
    print("Unexpected dynamic symbols in binary: {0}".format(human_readable_difference))
    sys.exit(-1)
