#!/usr/bin/env python

import os
import subprocess
import sys
import tempfile
import shutil
import bisect
import argparse

KHRONOS_REPO_URL = "https://github.com/KhronosGroup/WebGL.git"
# Patches for conformance tests 1.0.x
PATCHES = [
    ("js-test-pre.patch", "js/js-test-pre.js"),
    ("unit.patch", "conformance/more/unit.js"),
    ("timeout.patch", None),
    ("set-zero-timeout.patch", "js/webgl-test-utils.js"),
    ("compressed-images.patch", "conformance/extensions/webgl-compressed-texture-s3tc.html"),
    ("shader-varying-packing-restrictions.patch", "conformance/glsl/misc/shader-varying-packing-restrictions.html"),
]

# Fix for 'UnicodeDecodeError: 'ascii' codec can't decode byte'
reload(sys)  
sys.setdefaultencoding('utf8')

def usage():
    print("Usage: {} destination [existing_webgl_repo]".format(sys.argv[0]))
    sys.exit(1)

def get_tests(base_dir, file_name, tests_list):
    list_file = os.path.join(base_dir, file_name)
    if not os.path.isfile(list_file):
        print("Test list ({}) not found".format(list_file))
        sys.exit(1)

    print("Processing: {}".format(list_file))

    with open(list_file, "r") as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#") or line.startswith("//"):
                continue # It's an empty line or a comment

            # Lines often are in the form:
            # --min-version x.x.x abc.html
            #
            # We only care about the last part
            line = line.split(" ")[-1]

            if line.endswith(".html"):
                tests_list.append(os.path.join(base_dir, line))
            if line.endswith(".txt"):
                (next_dir, file_name) = os.path.split(os.path.join(base_dir, line))
                get_tests(next_dir, file_name, tests_list)


# Insert the test harness scripts before any other script
def process_test(test):
    (new, new_path) = tempfile.mkstemp()
    script_tag_found = False
    with open(test, "r") as test_file:
        for line in test_file:
            if not script_tag_found and "<script" in line:
                indent = ' ' * line.index('<')
                script_tag_found = True
                os.write(new, "{}<script src=/resources/testharness.js></script>\n".format(indent))
                os.write(new, "{}<script src=/resources/testharnessreport.js></script>\n".format(indent))
            os.write(new, line)

    os.close(new)
    shutil.move(new_path, test)



def update_conformance(destination, existing_repo, patches_dir):
    print("Trying to import WebGL tests into {}".format(destination))

    if existing_repo:
        directory = existing_repo
        print("Using existing WebGL repository: {}".format(directory))
    else:
        directory = tempfile.mkdtemp()
        print("Cloning WebGL repository into temporary directory {}".format(directory))
        subprocess.check_call(["git", "clone", KHRONOS_REPO_URL, directory, "--depth", "1"])

    suite_dir = os.path.join(directory, "sdk/tests")
    print("Test suite directory: {}".format(suite_dir))

    if not os.path.isdir(suite_dir):
        print("Test suite directory ({}) not found, aborting...".format(suite_dir))
        sys.exit(1)

    # We recursively copy all the test suite to `destination`
    shutil.copytree(suite_dir, destination)

    # Get all the tests, remove any html file which is not in the list, and
    # later process them.
    tests = []
    get_tests(destination, "00_test_list.txt", tests)

    test_count = len(tests)

    print("Found {} tests.".format(test_count))
    print("Removing non-test html files")

    # To use binary search, which speeds things up a little
    # instead of f in tests
    tests.sort()

    # Remove html files that are not tests
    for dirpath, dirnames, filenames in os.walk(destination):
        if '/resources' in dirpath:
          continue # Most of the files under resources directories are used

        for f in filenames:
            if not f.endswith('.html'):
                continue

            f = os.path.join(dirpath, f)
            pos = bisect.bisect_left(tests, f)
            if pos == test_count or tests[pos] != f:
                print("Removing: {}".format(f))
                os.remove(f)

    # Insert our harness into the tests
    for test in tests:
        process_test(test)

    # Try to apply the patches to the required files
    if not patches_dir:
        patches_dir = os.path.abspath(os.path.dirname(sys.argv[0]))
    for patch, file_name in PATCHES:
        try:
            patch = os.path.join(patches_dir, patch)
            if file_name is None:
                subprocess.check_call(["patch", "-d", destination, "-p", "1"], stdin=open(patch))
            else:
                subprocess.check_call(["patch", "-x", "3", "-d", destination, file_name, patch])
        except subprocess.CalledProcessError:
            print("Automatic patch failed for {}".format(file_name))
            print("Please review the WPT integration and update {} accordingly".format(os.path.basename(patch)))

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("destination", help="Test suite destination")
    parser.add_argument("-e", "--existing-repo", help="Path to an existing clone of the khronos WebGL repository")
    args = parser.parse_args()

    update_conformance(args.destination, args.existing_repo, None)

if __name__ == '__main__':
    main()
