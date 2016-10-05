import re

bindings = open("bindings.rs", "r")
tests    = open("test_bindings.rs", "w")

tests.write("fn assert_types() {\n")

pattern  = re.compile("fn\s*Servo_([a-zA-Z0-9]+)\s*\(")

for line in bindings:
    match = pattern.search(line);

    if match :
        tests.write("    [ Servo_" + match.group(1) + ", bindings::Servo_" + match.group(1) + " ];\n")

tests.write("}\n")

bindings.close()
tests.close()
