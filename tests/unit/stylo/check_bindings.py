#!/usr/bin/env python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import os
import re

ROOT_PATH = os.path.join("..", "..", "..")
INPUT_FILE = os.path.join(ROOT_PATH, "components", "style", "gecko", "generated", "bindings.rs")
OUTPUT_FILE = os.path.join(os.environ["OUT_DIR"], "check_bindings.rs")
GLUE_FILE = os.path.join(ROOT_PATH, "ports", "geckolib", "glue.rs")
GLUE_OUTPUT_FILE = os.path.join(os.environ["OUT_DIR"], "glue.rs")

TEMPLATE = """\
    [ Servo_{name}, bindings::Servo_{name} ];
"""

with open(INPUT_FILE, "r") as bindings, open(OUTPUT_FILE, "w+") as tests:
    tests.write("fn assert_types() {\n")

    pattern = re.compile("fn\s*Servo_([a-zA-Z0-9_]+)\s*\(")

    for line in bindings:
        match = pattern.search(line)
        if match:
            tests.write(TEMPLATE.format(name=match.group(1)))

    tests.write("}\n")

with open(GLUE_FILE, "r") as glue, open(GLUE_OUTPUT_FILE, "w+") as glue_output:
    glue_output.write("pub use style::gecko::arc_types::*;")
    for line in glue:
        glue_output.write(line.replace("pub extern \"C\" fn", "pub unsafe extern \"C\" fn"))
