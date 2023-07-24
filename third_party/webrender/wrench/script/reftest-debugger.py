#!/usr/bin/python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import print_function
import subprocess

with open('reftest.log', "w") as out:
    try:
        subprocess.check_call(['script/headless.py', 'reftest'], stdout=out)
        print("All tests passed.")
    except subprocess.CalledProcessError as ex:
        subprocess.check_call(['firefox', 'reftest-analyzer.xhtml#logurl=reftest.log'])
