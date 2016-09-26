import os
import re
import shutil
import glob
import tempfile
import sys
from collections import defaultdict

here = os.path.abspath(os.path.split(__file__)[0])

def get_extra_files(chromium_root):
    return [(os.path.join(chromium_root, "LayoutTests", "http", "tests", "resources", "testharness-helpers.js"),
             os.path.join("resources", "testharness-helpers.js"))]

resources_re = re.compile("/?(?:\.\./)*resources/(testharness(?:report)?)\.js")

def resources_path(line, depth):
    return False, resources_re.sub(r"/resources/\1.js", line)

php_re = re.compile("\.php")

def python_to_php(line, depth):
    return False, php_re.sub(".py", line)

abs_testharness_helpers_re = re.compile("([\"'])/resources/testharness-helpers.js")
testharness_helpers_re = re.compile("\.\./((?:\.\./)*)resources/testharness-helpers.js")

def testharness_helpers(line, depth):
    if abs_testharness_helpers_re.findall(line):
        return False, abs_testharness_helpers_re.sub(r"\1%sresources/testharness-helpers.js" % ("../" * (depth - 1)), line)
    return False, testharness_helpers_re.sub(r"\1resources/testharness-helpers.js", line)

serviceworker_path_re = re.compile("/serviceworker/")
def service_worker_path(line, depth):
    return False, serviceworker_path_re.sub("/service-workers/", line)

localhost_re = re.compile("localhost")
alt_host_re = re.compile("127\.0\.0\.1")
port_http_re = re.compile("8000")
port_https_re = re.compile("8000")


def server_names(line, depth):
    line, count_0 = localhost_re.subn("{{host}}", line)
    line, count_1 = alt_host_re.subn("{{domains[www]}}", line)
    line, count_2 = port_http_re.subn("{{ports[http][0]}}", line)
    line, count_3 = port_https_re.subn("{{ports[https][0]}}", line)

    count = count_0 + count_1 + count_2 + count_3

    return bool(count), line


def source_paths(chromium_root):
    for dirpath, dirnames, filenames in os.walk(chromium_root):
        if "chromium" in dirnames:
            dirnames.remove("chromium")
        for filename in filenames:
            if filename.endswith("-expected.txt") or filename.endswith(".php"):
                continue
            yield os.path.relpath(os.path.join(dirpath, filename), chromium_root)


def do_subs(path, line):
    depth = len(os.path.split(os.path.sep))
    subs = [resources_path, python_to_php, testharness_helpers, service_worker_path, server_names]
    file_is_template = False
    for sub in subs:
        added_template, line = sub(line, depth)
        if added_template:
            file_is_template = True
    return file_is_template, line

def get_head(git):
    return git("rev-parse", "HEAD")

def get_changes(git, path, old, new):
    data = git("diff", "--name-status", "-z", "--no-renames", "%s..%s" % (old, new), "--", path)
    items = data.split("\0")
    rv = defaultdict(list)
    for status, path in items:
        rv[status].append(path)

    return rv

def copy(src_path, out_dir, rel_path):
    dest = os.path.normpath(os.path.join(out_dir, rel_path))
    dest_dir = os.path.split(dest)[0]
    if not os.path.exists(dest_dir):
        os.makedirs(dest_dir)
    shutil.copy2(src_path, dest)

def copy_local_files(local_files, out_root, tmp_dir):
    for path in local_files:
        rel_path = os.path.relpath(path, out_root)
        copy(path, tmp_dir, rel_path)

def copy_extra_files(chromium_root, tmp_dir):
    for in_path, rel_path in get_extra_files(chromium_root):
        copy(in_path, tmp_dir, rel_path)

def sub_changed_filenames(filename_changes, f):
    rv = []
    for line in f:
        for in_name, out_name in filename_changes.iteritems():
            line = line.replace(in_name, out_name)
        rv.append(line)
    return "".join(rv)

testharness_re = re.compile("<script[^>]*src=[\"']?/resources/testharness.js[\"' ][^>]*>")

def is_top_level_test(path, data):
    if os.path.splitext(path)[1] != ".html":
        return False
    for line in data:
        if testharness_re.findall(line):
            return True
    return False

def add_suffix(path, suffix):
    root, ext = os.path.splitext(path)
    return root + ".%s" % suffix + ext

def main():
    if "--cache-tests" in sys.argv:
        sw_path = os.path.join("LayoutTests", "http", "tests", "cachestorage")
        out_root = os.path.abspath(os.path.join(here, "..", "cache-storage"))
    elif "--sw-tests" in sys.argv:
        sw_path = os.path.join("LayoutTests", "http", "tests", "serviceworkers")
        out_root = os.path.abspath(os.path.join(here, "..", "service-worker"))
    else:
        raise ValueError("Must supply either --cache-tests or --sw-tests")

    chromium_root = os.path.abspath(sys.argv[1])

    work_path = tempfile.mkdtemp()

    test_path = os.path.join(chromium_root, sw_path)

    local_files = glob.glob(os.path.normpath(os.path.join(here, "..", "resources", "*.py")))

    if not os.path.exists(out_root):
        os.mkdir(out_root)

    copy_local_files(local_files, out_root, work_path)
    copy_extra_files(chromium_root, work_path)

    path_changes = {}

    for path in source_paths(test_path):
        out_path = os.path.join(work_path, path)
        out_dir = os.path.dirname(out_path)
        if not os.path.exists(out_dir):
            os.makedirs(out_dir)
        with open(os.path.join(test_path, path), "r") as in_f:
            data = []
            sub = False
            for line in in_f:
                sub_flag, output_line = do_subs(path, line)
                data.append(output_line)
                if sub_flag:
                    sub = True
            is_test = is_top_level_test(out_path, data)

        initial_path = out_path

        if is_test:
            path_1 = add_suffix(out_path, "https")
        else:
            path_1 = out_path

        if sub:
            path_2 = add_suffix(out_path, "sub")
        else:
            path_2 = path_1

        if path_2 != initial_path:
            path_changes[initial_path] = path_2

        with open(path_2, "w") as out_f:
            out_f.write("".join(data))

    filename_changes = {}

    for k, v in path_changes.iteritems():
        if os.path.basename(k) in filename_changes:
            print "Got duplicate name:" + os.path.basename(k)
        filename_changes[os.path.basename(k)] = os.path.basename(v)

    for path in source_paths(work_path):
        full_path = os.path.join(work_path, path)
        with open(full_path, "r") as f:
            data = sub_changed_filenames(filename_changes, f)
        with open(full_path, "w") as f:
            f.write(data)

    for dirpath, dirnames, filenames in os.walk(work_path):
        for filename in filenames:
            in_path = os.path.join(dirpath, filename)
            rel_path = os.path.relpath(in_path, work_path)
            copy(in_path, out_root, rel_path)

if __name__ == "__main__":
    main()
