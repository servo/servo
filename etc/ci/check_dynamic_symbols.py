# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import os
import re
import subprocess
import sys

symbol_regex = re.compile(br"D  \*UND\*\t(.*) (.*)$")
allowed_symbols = frozenset([
    b'unshare',
    b'malloc_usable_size',
    b'__cxa_type_match',
    b'signal',
    b'tcgetattr',
    b'tcsetattr',
    b'__strncpy_chk2',
    b'rand',
    b'__read_chk',
    b'fesetenv',
    b'srand',
    b'abs',
    b'fegetenv',
    b'sigemptyset',
    b'AHardwareBuffer_allocate',
    b'AHardwareBuffer_release',
    b'getentropy',
])
actual_symbols = set()

objdump_output = subprocess.check_output([
    os.path.join(
        'android-toolchains', 'ndk', 'toolchains', 'arm-linux-androideabi-4.9',
        'prebuilt', 'linux-x86_64', 'bin', 'arm-linux-androideabi-objdump'),
    '-T',
    'target/android/armv7-linux-androideabi/debug/libservoshell.so']
).split(b'\n')

for line in objdump_output:
    m = symbol_regex.search(line)
    if m is not None:
        actual_symbols.add(m.group(2))

difference = actual_symbols - allowed_symbols

if len(difference) > 0:
    human_readable_difference = "\n".join(str(s) for s in difference)
    print("Unexpected dynamic symbols in binary:\n{0}".format(human_readable_difference))
    sys.exit(-1)
