# mypy: allow-untyped-defs

import subprocess
import os

here = os.path.dirname(__file__)

template_prefix = """<!doctype html>
%(documentElement)s<meta charset=utf-8>
"""
template_long_timeout = "<meta name=timeout content=long>\n"

template_body_th = """<title></title>
<script src=/resources/testharness.js></script>
<script src=/resources/testharnessreport.js></script>
<script>

</script>
"""

template_body_reftest = """<title></title>
<link rel=%(match)s href=%(ref)s>
"""

template_body_reftest_wait = """<script src="/common/reftest-wait.js"></script>
"""

def get_parser():
    import argparse
    p = argparse.ArgumentParser()
    p.add_argument("--no-editor", action="store_true",
                   help="Don't try to open the test in an editor")
    p.add_argument("-e", "--editor", help="Editor to use")
    p.add_argument("--long-timeout", action="store_true",
                   help="Test should be given a long timeout (typically 60s rather than 10s, but varies depending on environment)")
    p.add_argument("--overwrite", action="store_true",
                   help="Allow overwriting an existing test file")
    p.add_argument("-r", "--reftest", action="store_true",
                   help="Create a reftest rather than a testharness (js) test"),
    p.add_argument("-m", "--reference", dest="ref", help="Path to the reference file")
    p.add_argument("--mismatch", action="store_true",
                   help="Create a mismatch reftest")
    p.add_argument("--wait", action="store_true",
                   help="Create a reftest that waits until takeScreenshot() is called")
    p.add_argument("--tests-root", default=os.path.join(here, "..", ".."),
                   help="Path to the root of the wpt directory")
    p.add_argument("path", help="Path to the test file")
    return p



def rel_path(path, tests_root):
    if path is None:
        return

    abs_path = os.path.normpath(os.path.abspath(path))
    return os.path.relpath(abs_path, tests_root)


def run(_venv, **kwargs):
    path = rel_path(kwargs["path"], kwargs["tests_root"])
    ref_path = rel_path(kwargs["ref"], kwargs["tests_root"])

    if kwargs["ref"]:
        kwargs["reftest"] = True

    if ".." in path:
        print("""Test path %s is not under wpt root.""" % path)
        return 1

    if ref_path and ".." in ref_path:
        print("""Reference path %s is not under wpt root""" % ref_path)
        return 1


    if os.path.exists(path) and not kwargs["overwrite"]:
        print("Test path already exists, pass --overwrite to replace")
        return 1

    if kwargs["mismatch"] and not kwargs["reftest"]:
        print("--mismatch only makes sense for a reftest")
        return 1

    if kwargs["wait"] and not kwargs["reftest"]:
        print("--wait only makes sense for a reftest")
        return 1

    args = {"documentElement": "<html class=reftest-wait>\n" if kwargs["wait"] else ""}
    template = template_prefix % args
    if kwargs["long_timeout"]:
        template += template_long_timeout

    if kwargs["reftest"]:
        args = {"match": "match" if not kwargs["mismatch"] else "mismatch",
                "ref": os.path.relpath(ref_path, path) if kwargs["ref"] else '""'}
        template += template_body_reftest % args
        if kwargs["wait"]:
            template += template_body_reftest_wait
    else:
        template += template_body_th
    try:
        os.makedirs(os.path.dirname(path))
    except OSError:
        pass
    with open(path, "w") as f:
        f.write(template)

    ref_path = kwargs["ref"]
    if ref_path and not os.path.exists(ref_path):
        with open(ref_path, "w") as f:
            f.write(template_prefix % {"documentElement": ""})

    if kwargs["no_editor"]:
        editor = None
    elif kwargs["editor"]:
        editor = kwargs["editor"]
    elif "VISUAL" in os.environ:
        editor = os.environ["VISUAL"]
    elif "EDITOR" in os.environ:
        editor = os.environ["EDITOR"]
    else:
        editor = None

    proc = None
    if editor:
        if ref_path:
            path = f"{path} {ref_path}"
        proc = subprocess.Popen(f"{editor} {path}", shell=True)
    else:
        print("Created test %s" % path)

    if proc:
        proc.wait()
