#!/usr/bin/env python3
# mypy: allow-untyped-defs

import argparse
import gzip
import logging
import os
import shutil
import subprocess
import sys


def get_browser_args(product, channel, artifact_path):
    if product == "firefox":
        local_binary = os.path.expanduser(os.path.join("~", "build", "firefox", "firefox"))
        if os.path.exists(local_binary):
            return ["--binary=%s" % local_binary]
        print("WARNING: Local firefox binary not found")
        return ["--install-browser", "--install-webdriver"]
    if product == "firefox_android":
        return ["--install-browser", "--install-webdriver", "--logcat-dir", artifact_path]
    if product == "servo":
        return ["--install-browser", "--processes=12"]
    if product == "chrome" or product == "chromium":
        # Taskcluster machines do not have GPUs, so use software rendering via --enable-swiftshader.
        args = ["--enable-swiftshader"]
        if channel == "nightly":
            args.extend(["--install-browser", "--install-webdriver"])
        return args
    if product == "webkitgtk_minibrowser":
        # Using 4 parallel jobs gives 4x speed-up even on a 1-core machine and doesn't cause extra timeouts.
        # See: https://github.com/web-platform-tests/wpt/issues/38723#issuecomment-1470938179
        return ["--install-browser", "--processes=4"]
    return []


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


def main(product, channel, commit_range, artifact_path, wpt_args):
    """Invoke the `wpt run` command according to the needs of the Taskcluster
    continuous integration service."""

    logger = logging.getLogger("tc-run")
    logger.setLevel(logging.INFO)
    handler = logging.StreamHandler()
    handler.setFormatter(
        logging.Formatter("%(asctime)s - %(name)s - %(levelname)s - %(message)s")
    )
    logger.addHandler(handler)

    subprocess.call(['python3', './wpt', 'manifest-download'])

    if commit_range:
        logger.info(
            "Running tests affected in range '%s'..." % commit_range
        )
        wpt_args += ['--affected', commit_range]
    else:
        logger.info("Running all tests")

    wpt_args += [
        "--log-mach-level=info",
        "--log-mach=-",
        "-y",
        "--no-pause",
        "--no-restart-on-unexpected",
        "--install-fonts",
        "--no-headless",
        "--verify-log-full"
    ]
    wpt_args += get_browser_args(product, channel, artifact_path)

    # Hack to run servo with one process only for wdspec
    if product == "servo" and "--test-type=wdspec" in wpt_args:
        wpt_args = [item for item in wpt_args if not item.startswith("--processes")]

    wpt_args.append(product)

    command = ["python3", "./wpt", "run"] + wpt_args

    logger.info("Executing command: %s" % " ".join(command))
    with open(os.path.join(artifact_path, "checkrun.md"), "a") as f:
        f.write("\n**WPT Command:** `%s`\n\n" % " ".join(command))

    retcode = subprocess.call(command, env=dict(os.environ, TERM="dumb"))
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
    parser.add_argument("--artifact-path", action="store",
                        default="/home/test/artifacts/",
                        help="Path to store output files")
    parser.add_argument("product", action="store",
                        help="Browser to run tests in")
    parser.add_argument("channel", action="store",
                        help="Channel of the browser")
    parser.add_argument("wpt_args", nargs="*",
                        help="Arguments to forward to `wpt run` command")
    main(**vars(parser.parse_args()))  # type: ignore
