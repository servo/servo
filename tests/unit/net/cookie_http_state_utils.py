# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import os
import re
import subprocess

REPO = "https://github.com/abarth/http-state.git"
TEST_FILE = "cookie_http_state.rs"
DOMAIN = "http://home.example.org:8888"
RUST_FN = """
#[test]
fn test_{name}(){{
    let result = run_test(
            "{set_location}",
            {set_cookies},
            "{location}");
    assert_eq!(result, "{expect}".to_string());
}}
"""


def list_tests(dir):
    def keep(name):
        return name.endswith("-test") and not name.startswith("disabled")
    tests = [name[:-5] for name in os.listdir(dir) if keep(name)]
    tests.sort()
    return tests


def escape(s):
    """ Escape the string `s` so that it can be parsed by rust as a valid
    UTF-8 string.
    We can't use only `encode("unicode_escape")` as it produces things that
    rust does not accept ("\\xbf", "\\u6265" for example). So we manually
    convert all character whose code point is greater than 128 to
    \\u{code_point}.
    All other characters are encoded with "unicode_escape" to get escape
    sequences ("\\r" for example) except for `"` that we specifically escape
    because our string will be quoted by double-quotes.
    """
    res = ""
    for c in s:
        o = ord(c)
        if o == 34:
            res += "\\\""
            continue
        if o >= 128:
            res += "\\u{" + hex(o)[2:] + "}"
        else:
            res += c.encode("unicode_escape")
    return res


def generate_code_for_test(test_dir, name):
    test_file = os.path.join(test_dir, name + "-test")
    expect_file = os.path.join(test_dir, name + "-expected")

    set_cookies = []
    set_location = DOMAIN + "/cookie-parser?" + name
    expect = ""
    location = DOMAIN + "/cookie-parser-result?" + name

    with open(test_file) as fo:
        for line in fo:
            line = line.decode("utf-8").rstrip()
            if line.startswith("Set-Cookie: "):
                set_cookies.append(line[12:])
                continue
            if line.startswith("Location: "):
                location = line[10:]
                if location.startswith("/"):
                    location = DOMAIN + location

    with open(expect_file) as fo:
        for line in fo:
            line = line.decode("utf-8").rstrip()
            if line.startswith("Cookie: "):
                expect = line[8:]
                continue
            if line.startswith("Received Cookie: "):
                expect = line[17:]

    fmt_set_cookies = ",".join('"%s"' % escape(c) for c in set_cookies)
    fmt_set_cookies = "&[" + fmt_set_cookies + "]"

    return RUST_FN.format(
            name=name.replace('-', '_'),
            set_location=escape(set_location),
            set_cookies=fmt_set_cookies,
            location=escape(location),
            expect=escape(expect))


def update_test_file(cachedir):
    workdir = os.path.dirname(os.path.realpath(__file__))
    test_file = os.path.join(workdir, TEST_FILE)

    # Create the cache dir
    if not os.path.isdir(cachedir):
        os.makedirs(cachedir)

    # Clone or update the repo
    repo_dir = os.path.join(cachedir, "http-state")
    if os.path.isdir(repo_dir):
        args = ["git", "pull", "-f"]
        process = subprocess.Popen(args, cwd=repo_dir)
        if process.wait() != 0:
            print("failed to update the http-state git repo")
            return 1
    else:
        args = ["git", "clone", REPO, repo_dir]
        process = subprocess.Popen(args)
        if process.wait() != 0:
            print("failed to clone the http-state git repo")
            return 1

    # Truncate the unit test file to remove all existing tests
    with open(test_file, "r+") as fo:
        while True:
            line = fo.readline()
            if line.strip() == "// Test listing":
                fo.truncate()
                fo.flush()
                break
            if line == "":
                print("Failed to find listing delimiter on unit test file")
                return 1

    # Append all tests to unit test file
    tests_dir = os.path.join(repo_dir, "tests/data/parser")
    with open(test_file, "a") as fo:
        for test in list_tests(tests_dir):
            fo.write(generate_code_for_test(tests_dir, test).encode("utf-8"))

    return 0

if __name__ == "__main__":
    update_test_file("/tmp")
