#!/usr/bin/env python

import argparse
import gzip
import logging
import os
import shutil
import subprocess

browser_specific_args = {
    "firefox": ["--install-browser"]
}

def tests_affected(commit_range):
    output = subprocess.check_output([
        "python", "./wpt", "tests-affected", "--null", commit_range
    ], stderr=open(os.devnull, "w"))

    tests = output.split("\0")

    # Account for trailing null byte
    if tests and not tests[-1]:
        tests.pop()

    return tests


def find_wptreport(args):
    parser = argparse.ArgumentParser()
    parser.add_argument('--log-wptreport', action='store')
    return parser.parse_known_args(args)[0].log_wptreport


def gzip_file(filename):
    with open(filename, 'rb') as f_in:
        with gzip.open('%s.gz' % filename, 'wb') as f_out:
            shutil.copyfileobj(f_in, f_out)


def main(product, commit_range, wpt_args):
    """Invoke the `wpt run` command according to the needs of the TaskCluster
    continuous integration service."""

    logger = logging.getLogger("tc-run")
    logger.setLevel(logging.INFO)
    handler = logging.StreamHandler()
    handler.setFormatter(
        logging.Formatter("%(asctime)s - %(name)s - %(levelname)s - %(message)s")
    )
    logger.addHandler(handler)

    child = subprocess.Popen(['python', './wpt', 'manifest-download'])
    child.wait()

    if commit_range:
        logger.info(
            "Identifying tests affected in range '%s'..." % commit_range
        )
        tests = tests_affected(commit_range)
        logger.info("Identified %s affected tests" % len(tests))

        if not tests:
            logger.info("Quitting because no tests were affected.")
            return
    else:
        tests = []
        logger.info("Running all tests")

    wpt_args += [
        "--log-tbpl=../artifacts/log_tbpl.log",
        "--log-tbpl-level=info",
        "--log-mach=-",
        "-y",
        "--no-pause",
        "--no-restart-on-unexpected",
        "--install-fonts",
        "--no-headless"
    ]
    wpt_args += browser_specific_args.get(product, [])

    command = ["python", "./wpt", "run"] + wpt_args + [product] + tests

    logger.info("Executing command: %s" % " ".join(command))

    subprocess.check_call(command)

    wptreport = find_wptreport(wpt_args)
    if wptreport:
        gzip_file(wptreport)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=main.__doc__)
    parser.add_argument("--commit-range", action="store",
                        help="""Git commit range. If specified, this will be
                             supplied to the `wpt tests-affected` command to
                             determine the list of test to execute""")
    parser.add_argument("product", action="store",
                        help="Browser to run tests in")
    parser.add_argument("wpt_args", nargs="*",
                        help="Arguments to forward to `wpt run` command")
    main(**vars(parser.parse_args()))
