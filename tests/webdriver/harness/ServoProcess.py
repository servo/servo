# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import subprocess


class ServoProcess(object):
    def __init__(self):
        self.path = "path/to/servo"
        self.proc = None

    def __enter__(self):
        self.proc = subprocess.Popen(["./mach run --webdriver 7000 tests/html/about-mozilla.html"], shell=True)

    def __exit__(self, *args, **kwargs):
        self.proc.kill()
