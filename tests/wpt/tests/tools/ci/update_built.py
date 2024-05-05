# mypy: allow-untyped-defs

import logging
import os
import subprocess
from argparse import ArgumentParser

logger = logging.getLogger()

wpt_root = os.path.abspath(os.path.join(os.path.dirname(__file__), os.pardir, os.pardir))

# These paths should be kept in sync with job_path_map in jobs.py.
scripts = {
    "canvas": ["html/canvas/tools/gentest.py"],
    "conformance-checkers": ["conformance-checkers/tools/dl.py",
                             "conformance-checkers/tools/ins-del-datetime.py",
                             "conformance-checkers/tools/picture.py",
                             "conformance-checkers/tools/url.py"],
    "css-images": ["css/css-images/tools/generate_object_view_box_tests.py"],
    "css-ui": ["css/css-ui/tools/appearance-build-webkit-reftests.py"],
    "css-writing-modes": ["css/css-writing-modes/tools/generators/generate.py"],
    # FIXME: https://github.com/web-platform-tests/wpt/issues/32060
    # "css-text": ["css/css-text/line-breaking/tools/generate-segment-break-transformation-rules-tests.py"],
    # "css-text-decor": ["css/css-text-decor/tools/generate-text-emphasis-line-height-tests.py",
    #                    "css/css-text-decor/tools/generate-text-emphasis-position-property-tests.py",
    #                    "css/css-text-decor/tools/generate-text-emphasis-ruby-tests.py",
    #                    "css/css-text-decor/tools/generate-text-emphasis-style-property-tests.py"],
    "fetch": ["fetch/metadata/tools/generate.py"],
    "html5lib": ["html/tools/update_html5lib_tests.py"],
    "infrastructure": ["infrastructure/assumptions/tools/ahem-generate-table.py"],
    "mimesniff": ["mimesniff/mime-types/resources/generated-mime-types.py"],
    "speculative-parsing": ["html/syntax/speculative-parsing/tools/generate.py"]
}


def get_parser():
    parser = ArgumentParser()
    parser.add_argument("--list", action="store_true",
                        help="List suites that can be updated and the related script files")
    parser.add_argument("--include", nargs="*", choices=scripts.keys(), default=None,
                        help="Suites to update (default is to update everything)")
    return parser


def list_suites(include):
    for name, script_paths in scripts.items():
        if name in include:
            print(name)
            for script_path in script_paths:
                print(f"    {script_path}")


def run(venv, **kwargs):
    include = kwargs["include"]
    if include is None:
        include = list(scripts.keys())

    if kwargs["list"]:
        list_suites(include)
        return 0

    failed = False

    for target in include:
        for script in scripts[target]:
            script_path = script.replace("/", os.path.sep)
            cmd = [os.path.join(venv.bin_path, "python3"), os.path.join(wpt_root, script_path)]
            logger.info(f"Running {' '.join(cmd)}")
            try:
                subprocess.check_call(cmd, cwd=os.path.dirname(script_path))
            except subprocess.CalledProcessError:
                logger.error(f"Update script {script} failed")
                failed = True

    return 1 if failed else 0
