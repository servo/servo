import glob
import hashlib
import itertools
import json
import os
import re
import shutil
import site
import subprocess
import sys
import tempfile
import urllib
from importlib import reload


import genshi
from genshi.template import MarkupTemplate


TESTS_PATH = "html/syntax/parsing/"

def get_paths():
    script_path = os.path.dirname(os.path.abspath(__file__))
    repo_base = get_repo_base(script_path)
    tests_path = os.path.join(repo_base, TESTS_PATH)
    return script_path, tests_path


def get_repo_base(path):
    while path:
        if os.path.exists(os.path.join(path, ".git")):
            return path
        else:
            path = os.path.dirname(path)


def get_expected(data):
    data = "#document\n" + data
    return data


def get_hash(data, container=None):
    if container == None:
        container = ""
    return hashlib.sha1(b"#container%s#data%s"%(container.encode("utf8"),
                                               data.encode("utf8"))).hexdigest()


class Html5libInstall:
    def __init__(self, rev=None):
        self.html5lib_dir = None
        self.rev = rev

    def __enter__(self):
        self.html5lib_dir = tempfile.TemporaryDirectory()
        html5lib_path = self.html5lib_dir.__enter__()
        subprocess.check_call(["git", "clone", "--no-checkout", "https://github.com/html5lib/html5lib-python.git", "html5lib"],
                              cwd=html5lib_path)
        rev = self.rev if self.rev is not None else "origin/master"
        subprocess.check_call(["git", "checkout", rev],
                              cwd=os.path.join(html5lib_path, "html5lib"))
        subprocess.check_call(["pip", "install", "-e", "html5lib"], cwd=html5lib_path)
        reload(site)

    def __exit__(self, *args, **kwargs):
        subprocess.call(["pip", "uninstall", "-y", "html5lib"], cwd=self.html5lib_dir.name)
        self.html5lib_dir.__exit__(*args, **kwargs)
        self.html5lib_dir = None


def make_tests(script_dir, out_dir, input_file_name, test_data):
    tests = []
    innerHTML_tests = []
    ids_seen = {}
    print(input_file_name)
    for test in test_data:
        if "script-off" in test:
            continue
        is_innerHTML = "document-fragment" in test
        data = test["data"]
        container = test["document-fragment"] if is_innerHTML else None
        assert test["document"], test
        expected = get_expected(test["document"])
        test_list = innerHTML_tests if is_innerHTML else tests
        test_id = get_hash(data, container)
        if test_id in ids_seen:
            print("WARNING: id %s seen multiple times in file %s this time for test (%s, %s) before for test %s, skipping"%(test_id, input_file_name, container, data, ids_seen[test_id]))
            continue
        ids_seen[test_id] = (container, data)
        test_list.append({'string_uri_encoded_input':"\"%s\""%urllib.parse.quote(data.encode("utf8")),
                          'input':data,
                          'expected':expected,
                          'string_escaped_expected':json.dumps(urllib.parse.quote(expected.encode("utf8"))),
                          'id':test_id,
                          'container':container
                          })
    path_normal = None
    if tests:
        path_normal = write_test_file(script_dir, out_dir,
                                      tests, "html5lib_%s"%input_file_name,
                                      "html5lib_test.xml")
    path_innerHTML = None
    if innerHTML_tests:
        path_innerHTML = write_test_file(script_dir, out_dir,
                                         innerHTML_tests, "html5lib_innerHTML_%s"%input_file_name,
                                         "html5lib_test_fragment.xml")

    return path_normal, path_innerHTML

def write_test_file(script_dir, out_dir, tests, file_name, template_file_name):
    file_name = os.path.join(out_dir, file_name + ".html")
    short_name = os.path.basename(file_name)

    with open(os.path.join(script_dir, template_file_name), "r") as f:
        template = MarkupTemplate(f)

    stream = template.generate(file_name=short_name, tests=tests)

    with open(file_name, "w") as f:
        f.write(str(stream.render('html', doctype='html5',
                              encoding="utf8"), "utf-8"))
    return file_name

def escape_js_string(in_data):
    return in_data.encode("utf8").encode("string-escape")

def serialize_filenames(test_filenames):
    return "[" + ",\n".join("\"%s\""%item for item in test_filenames) + "]"

def main():
    script_dir, out_dir = get_paths()

    test_files = []
    inner_html_files = []
    with open(os.path.join(script_dir, "html5lib_revision"), "r") as f:
        html5lib_rev = f.read().strip()

    with Html5libInstall(html5lib_rev):
        from html5lib.tests import support

        if len(sys.argv) > 2:
            test_iterator = zip(
                itertools.repeat(False),
                sorted(os.path.abspath(item) for item in
                       glob.glob(os.path.join(sys.argv[2], "*.dat"))))
        else:
            test_iterator = itertools.chain(
                zip(itertools.repeat(False),
                               sorted(support.get_data_files("tree-construction"))),
                zip(itertools.repeat(True),
                               sorted(support.get_data_files(
                            os.path.join("tree-construction", "scripted")))))

        for (scripted, test_file) in test_iterator:
            input_file_name = os.path.splitext(os.path.basename(test_file))[0]
            if scripted:
                input_file_name = "scripted_" + input_file_name
            test_data = support.TestData(test_file)
            test_filename, inner_html_file_name = make_tests(script_dir, out_dir,
                                                             input_file_name, test_data)
            if test_filename is not None:
                test_files.append(test_filename)
            if inner_html_file_name is not None:
                inner_html_files.append(inner_html_file_name)

if __name__ == "__main__":
    main()
