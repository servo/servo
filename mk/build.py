# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import os
import subprocess


def main():
    print("Create build/")
    os.mkdir("build")

    commands = [
        ["sh", "configure"],
        ["make", "tidy"],
        ["make", "-j", "2"]
    ]

    for command in commands:
        print("Running %s" % " ".join(command))
        subprocess.check_call(command, cwd="build")


if __name__ == "__main__":
    main()
