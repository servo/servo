# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import re

bindings = open("bindings.rs", "r")
tests = open("test_bindings.rs", "w")

tests.write("/* This Source Code Form is subject to the terms of the Mozilla Public\n")
tests.write(" * License, v. 2.0. If a copy of the MPL was not distributed with this\n")
tests.write(" * file, You can obtain one at http://mozilla.org/MPL/2.0/. */\n\n")
tests.write("fn assert_types() {\n")

pattern = re.compile("fn\s*Servo_([a-zA-Z0-9]+)\s*\(")

for line in bindings:
    match = pattern.search(line)

    if match:
        tests.write("    [ Servo_" + match.group(1) + ", bindings::Servo_" + match.group(1) + " ];\n")

tests.write("}\n")

bindings.close()
tests.close()
