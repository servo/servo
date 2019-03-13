#!/usr/bin/env python

import argparse
import gzip
import logging
import os
import shutil
import subprocess
import sys

browser_specific_args = {
    "firefox": ["--install-browser"]
}


def find_wptreport(args):
    parser = argparse.ArgumentParser()
    parser.add_argument('--log-wptreport', action='store')
    return parser.parse_known_args(args)[0].log_wptreport


def find_wptscreenshot(args):
    parser = argparse.ArgumentParser()
    parser.add_argument('--log-wptscreenshot', action='store')
    return parser.parse_known_args(args)[0].log_wptscreenshot


def gzip_file(filename, delete_original=True):
    with open(filename, 'rb') as f_in:
        with gzip.open('%s.gz' % filename, 'wb') as f_out:
            shutil.copyfileobj(f_in, f_out)
    if delete_original:
        os.unlink(filename)


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

    subprocess.call(['python', './wpt', 'manifest-download'])

    if commit_range:
        logger.info(
            "Running tests affected in range '%s'..." % commit_range
        )
        wpt_args += ['--affected', commit_range]
    else:
        logger.info("Running all tests")

    wpt_args += [
        "--log-tbpl-level=info",
        "--log-tbpl=-",
        "-y",
        "--no-pause",
        "--no-restart-on-unexpected",
        "--install-fonts",
        "--no-headless",
        "--verify-log-full"
    ]
    wpt_args += browser_specific_args.get(product, [])

    command = ["python", "./wpt", "run"] + wpt_args + [product]

    logger.info("Executing command: %s" % " ".join(command))

    retcode = subprocess.call(command)
    if retcode != 0:
        sys.exit(retcode)

    wptreport = find_wptreport(wpt_args)
    if wptreport:
        gzip_file(wptreport)
    wptscreenshot = find_wptscreenshot(wpt_args)
    if wptscreenshot:
        gzip_file(wptscreenshot)


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
