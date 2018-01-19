# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import os
import subprocess
import tempfile

REPO = "https://github.com/abarth/http-state.git"
TEST_FILE = "cookie_http_state.rs"
DOMAIN = "http://home.example.org:8888"
RUST_FN = """
#[test]{should_panic}
fn test_{name}() {{
    let r = run("{set_location}",
                {set_cookies},
                "{location}");
    assert_eq!(&r, "{expect}");
}}
"""
SET_COOKIES_INDENT = 18
SHOULD_PANIC = "\n#[should_panic] // Look at cookie_http_state_utils.py if this test fails"

# Those tests should PASS. But until fixes land in servo, keep them failing
FAILING_TESTS = [
    "0003",           # Waiting for a way to clean expired cookies
    "0006",           # Waiting for a way to clean expired cookies
    "mozilla0001",    # Waiting for a way to clean expired cookies
    "mozilla0002",    # Waiting for a way to clean expired cookies
    "mozilla0003",    # Waiting for a way to clean expired cookies
    "mozilla0005",    # Waiting for a way to clean expired cookies
    "mozilla0007",    # Waiting for a way to clean expired cookies
    "mozilla0009",    # Waiting for a way to clean expired cookies
    "mozilla0010",    # Waiting for a way to clean expired cookies
    "mozilla0013",    # Waiting for a way to clean expired cookies
]


def list_tests(dir):
    suffix = "-test"

    def keep(name):
        return name.endswith(suffix) and not name.startswith("disabled")

    tests = [name[:-len(suffix)] for name in os.listdir(dir) if keep(name)]
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
    Lines are also limited in size, so split the string every 70 characters
    (gives room for indentation).
    """
    res = ""
    last_split = 0
    for c in s:
        if len(res) - last_split > 70:
            res += "\\\n"
            last_split = len(res)
        o = ord(c)
        if o == 34:
            res += "\\\""
            continue
        if o >= 128:
            res += "\\u{" + hex(o)[2:] + "}"
        else:
            res += c.encode("unicode_escape")
    return res


def format_slice_cookies(cookies):
    esc_cookies = ['"%s"' % escape(c) for c in cookies]
    if sum(len(s) for s in esc_cookies) < 80:
        sep = ", "
    else:
        sep = ",\n" + " " * SET_COOKIES_INDENT
    return "&[" + sep.join(esc_cookies) + "]"


def generate_code_for_test(test_dir, name):
    if name in FAILING_TESTS:
        should_panic = SHOULD_PANIC
    else:
        should_panic = ""

    test_file = os.path.join(test_dir, name + "-test")
    expect_file = os.path.join(test_dir, name + "-expected")

    set_cookies = []
    set_location = DOMAIN + "/cookie-parser?" + name
    expect = ""
    location = DOMAIN + "/cookie-parser-result?" + name

    with open(test_file) as fo:
        for line in fo:
            line = line.decode("utf-8").rstrip()
            prefix = "Set-Cookie: "
            if line.startswith(prefix):
                set_cookies.append(line[len(prefix):])
            prefix = "Location: "
            if line.startswith(prefix):
                location = line[len(prefix):]
                if location.startswith("/"):
                    location = DOMAIN + location

    with open(expect_file) as fo:
        for line in fo:
            line = line.decode("utf-8").rstrip()
            prefix = "Cookie: "
            if line.startswith(prefix):
                expect = line[len(prefix):]

    return RUST_FN.format(name=name.replace('-', '_'),
                          set_location=escape(set_location),
                          set_cookies=format_slice_cookies(set_cookies),
                          should_panic=should_panic,
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
    tests_dir = os.path.join(repo_dir, "tests", "data", "parser")
    with open(test_file, "a") as fo:
        for test in list_tests(tests_dir):
            fo.write(generate_code_for_test(tests_dir, test).encode("utf-8"))

    return 0

if __name__ == "__main__":
    update_test_file(tempfile.gettempdir())
