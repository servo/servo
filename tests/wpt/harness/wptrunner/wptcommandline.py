# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import argparse
import ast
import os
import sys
from collections import OrderedDict
from distutils.spawn import find_executable

import config
import wpttest


def abs_path(path):
    return os.path.abspath(os.path.expanduser(path))


def url_or_path(path):
    import urlparse

    parsed = urlparse.urlparse(path)
    if len(parsed.scheme) > 2:
        return path
    else:
        return abs_path(path)


def require_arg(kwargs, name, value_func=None):
    if value_func is None:
        value_func = lambda x: x is not None

    if not name in kwargs or not value_func(kwargs[name]):
        print >> sys.stderr, "Missing required argument %s" % name
        sys.exit(1)


def create_parser(product_choices=None):
    from mozlog import commandline

    import products

    if product_choices is None:
        config_data = config.load()
        product_choices = products.products_enabled(config_data)

    parser = argparse.ArgumentParser(description="""Runner for web-platform-tests tests.""",
                                     usage="""%(prog)s [OPTION]... [TEST]...

TEST is either the full path to a test file to run, or the URL of a test excluding
scheme host and port.""")
    parser.add_argument("--manifest-update", action="store_true", default=False,
                        help="Regenerate the test manifest.")

    parser.add_argument("--timeout-multiplier", action="store", type=float, default=None,
                        help="Multiplier relative to standard test timeout to use")
    parser.add_argument("--run-by-dir", type=int, nargs="?", default=False,
                        help="Split run into groups by directories. With a parameter,"
                        "limit the depth of splits e.g. --run-by-dir=1 to split by top-level"
                        "directory")
    parser.add_argument("--processes", action="store", type=int, default=None,
                        help="Number of simultaneous processes to use")

    parser.add_argument("--no-capture-stdio", action="store_true", default=False,
                        help="Don't capture stdio and write to logging")

    mode_group = parser.add_argument_group("Mode")
    mode_group.add_argument("--list-test-groups", action="store_true",
                            default=False,
                            help="List the top level directories containing tests that will run.")
    mode_group.add_argument("--list-disabled", action="store_true",
                            default=False,
                            help="List the tests that are disabled on the current platform")

    test_selection_group = parser.add_argument_group("Test Selection")
    test_selection_group.add_argument("--test-types", action="store",
                                      nargs="*", default=wpttest.enabled_tests,
                                      choices=wpttest.enabled_tests,
                                      help="Test types to run")
    test_selection_group.add_argument("--include", action="append",
                                      help="URL prefix to include")
    test_selection_group.add_argument("--exclude", action="append",
                                      help="URL prefix to exclude")
    test_selection_group.add_argument("--include-manifest", type=abs_path,
                                      help="Path to manifest listing tests to include")
    test_selection_group.add_argument("--tag", action="append", dest="tags",
                                      help="Labels applied to tests to include in the run. Labels starting dir: are equivalent to top-level directories.")

    debugging_group = parser.add_argument_group("Debugging")
    debugging_group.add_argument('--debugger', const="__default__", nargs="?",
                                 help="run under a debugger, e.g. gdb or valgrind")
    debugging_group.add_argument('--debugger-args', help="arguments to the debugger")
    debugging_group.add_argument("--repeat", action="store", type=int, default=1,
                                 help="Number of times to run the tests")
    debugging_group.add_argument("--repeat-until-unexpected", action="store_true", default=None,
                                 help="Run tests in a loop until one returns an unexpected result")
    debugging_group.add_argument('--pause-after-test', action="store_true", default=None,
                                 help="Halt the test runner after each test (this happens by default if only a single test is run)")
    debugging_group.add_argument('--no-pause-after-test', dest="pause_after_test", action="store_false",
                                 help="Don't halt the test runner irrespective of the number of tests run")

    debugging_group.add_argument('--pause-on-unexpected', action="store_true",
                                 help="Halt the test runner when an unexpected result is encountered")

    debugging_group.add_argument("--symbols-path", action="store", type=url_or_path,
                                 help="Path or url to symbols file used to analyse crash minidumps.")
    debugging_group.add_argument("--stackwalk-binary", action="store", type=abs_path,
                                 help="Path to stackwalker program used to analyse minidumps.")

    debugging_group.add_argument("--pdb", action="store_true",
                                 help="Drop into pdb on python exception")

    config_group = parser.add_argument_group("Configuration")
    config_group.add_argument("--binary", action="store",
                              type=abs_path, help="Binary to run tests against")
    config_group.add_argument('--binary-arg',
                              default=[], action="append", dest="binary_args",
                              help="Extra argument for the binary (servo)")
    config_group.add_argument("--webdriver-binary", action="store", metavar="BINARY",
                              type=abs_path, help="WebDriver server binary to use")

    config_group.add_argument("--metadata", action="store", type=abs_path, dest="metadata_root",
                              help="Path to root directory containing test metadata"),
    config_group.add_argument("--tests", action="store", type=abs_path, dest="tests_root",
                              help="Path to root directory containing test files"),
    config_group.add_argument("--run-info", action="store", type=abs_path,
                              help="Path to directory containing extra json files to add to run info")
    config_group.add_argument("--product", action="store", choices=product_choices,
                              default=None, help="Browser against which to run tests")
    config_group.add_argument("--config", action="store", type=abs_path, dest="config",
                              help="Path to config file")

    build_type = parser.add_mutually_exclusive_group()
    build_type.add_argument("--debug-build", dest="debug", action="store_true",
                            default=None,
                            help="Build is a debug build (overrides any mozinfo file)")
    build_type.add_argument("--release-build", dest="debug", action="store_false",
                            default=None,
                            help="Build is a release (overrides any mozinfo file)")


    chunking_group = parser.add_argument_group("Test Chunking")
    chunking_group.add_argument("--total-chunks", action="store", type=int, default=1,
                                help="Total number of chunks to use")
    chunking_group.add_argument("--this-chunk", action="store", type=int, default=1,
                                help="Chunk number to run")
    chunking_group.add_argument("--chunk-type", action="store", choices=["none", "equal_time", "hash"],
                                default=None, help="Chunking type to use")

    ssl_group = parser.add_argument_group("SSL/TLS")
    ssl_group.add_argument("--ssl-type", action="store", default=None,
                        choices=["openssl", "pregenerated", "none"],
                        help="Type of ssl support to enable (running without ssl may lead to spurious errors)")

    ssl_group.add_argument("--openssl-binary", action="store",
                        help="Path to openssl binary", default="openssl")
    ssl_group.add_argument("--certutil-binary", action="store",
                        help="Path to certutil binary for use with Firefox + ssl")

    ssl_group.add_argument("--ca-cert-path", action="store", type=abs_path,
                        help="Path to ca certificate when using pregenerated ssl certificates")
    ssl_group.add_argument("--host-key-path", action="store", type=abs_path,
                        help="Path to host private key when using pregenerated ssl certificates")
    ssl_group.add_argument("--host-cert-path", action="store", type=abs_path,
                        help="Path to host certificate when using pregenerated ssl certificates")

    gecko_group = parser.add_argument_group("Gecko-specific")
    gecko_group.add_argument("--prefs-root", dest="prefs_root", action="store", type=abs_path,
                             help="Path to the folder containing browser prefs")
    gecko_group.add_argument("--disable-e10s", dest="gecko_e10s", action="store_false", default=True,
                             help="Run tests without electrolysis preferences")

    b2g_group = parser.add_argument_group("B2G-specific")
    b2g_group.add_argument("--b2g-no-backup", action="store_true", default=False,
                           help="Don't backup device before testrun with --product=b2g")

    servo_group = parser.add_argument_group("Servo-specific")
    servo_group.add_argument("--user-stylesheet",
                             default=[], action="append", dest="user_stylesheets",
                             help="Inject a user CSS stylesheet into every test.")
    servo_group.add_argument("--servo-backend",
                             default="cpu", choices=["cpu", "webrender"],
                             help="Rendering backend to use with Servo.")


    parser.add_argument("test_list", nargs="*",
                        help="List of URLs for tests to run, or paths including tests to run. "
                             "(equivalent to --include)")

    commandline.add_logging_group(parser)
    return parser


def set_from_config(kwargs):
    if kwargs["config"] is None:
        config_path = config.path()
    else:
        config_path = kwargs["config"]

    kwargs["config_path"] = config_path

    kwargs["config"] = config.read(kwargs["config_path"])

    keys = {"paths": [("prefs", "prefs_root", True),
                      ("run_info", "run_info", True)],
            "web-platform-tests": [("remote_url", "remote_url", False),
                                   ("branch", "branch", False),
                                   ("sync_path", "sync_path", True)],
            "SSL": [("openssl_binary", "openssl_binary", True),
                    ("certutil_binary", "certutil_binary", True),
                    ("ca_cert_path", "ca_cert_path", True),
                    ("host_cert_path", "host_cert_path", True),
                    ("host_key_path", "host_key_path", True)]}

    for section, values in keys.iteritems():
        for config_value, kw_value, is_path in values:
            if kw_value in kwargs and kwargs[kw_value] is None:
                if not is_path:
                    new_value = kwargs["config"].get(section, config.ConfigDict({})).get(config_value)
                else:
                    new_value = kwargs["config"].get(section, config.ConfigDict({})).get_path(config_value)
                kwargs[kw_value] = new_value

    kwargs["test_paths"] = get_test_paths(kwargs["config"])

    if kwargs["tests_root"]:
        if "/" not in kwargs["test_paths"]:
            kwargs["test_paths"]["/"] = {}
        kwargs["test_paths"]["/"]["tests_path"] = kwargs["tests_root"]

    if kwargs["metadata_root"]:
        if "/" not in kwargs["test_paths"]:
            kwargs["test_paths"]["/"] = {}
        kwargs["test_paths"]["/"]["metadata_path"] = kwargs["metadata_root"]

    kwargs["suite_name"] = kwargs["config"].get("web-platform-tests", {}).get("name", "web-platform-tests")


def get_test_paths(config):
    # Set up test_paths
    test_paths = OrderedDict()

    for section in config.iterkeys():
        if section.startswith("manifest:"):
            manifest_opts = config.get(section)
            url_base = manifest_opts.get("url_base", "/")
            test_paths[url_base] = {
                "tests_path": manifest_opts.get_path("tests"),
                "metadata_path": manifest_opts.get_path("metadata")}

    return test_paths


def exe_path(name):
    if name is None:
        return

    path = find_executable(name)
    if os.access(path, os.X_OK):
        return path


def check_args(kwargs):
    set_from_config(kwargs)

    for test_paths in kwargs["test_paths"].itervalues():
        if not ("tests_path" in test_paths and
                "metadata_path" in test_paths):
            print "Fatal: must specify both a test path and metadata path"
            sys.exit(1)
        for key, path in test_paths.iteritems():
            name = key.split("_", 1)[0]

            if not os.path.exists(path):
                print "Fatal: %s path %s does not exist" % (name, path)
                sys.exit(1)

            if not os.path.isdir(path):
                print "Fatal: %s path %s is not a directory" % (name, path)
                sys.exit(1)

    if kwargs["product"] is None:
        kwargs["product"] = "firefox"

    if kwargs["test_list"]:
        if kwargs["include"] is not None:
            kwargs["include"].extend(kwargs["test_list"])
        else:
            kwargs["include"] = kwargs["test_list"]

    if kwargs["run_info"] is None:
        kwargs["run_info"] = kwargs["config_path"]

    if kwargs["this_chunk"] > 1:
        require_arg(kwargs, "total_chunks", lambda x: x >= kwargs["this_chunk"])

    if kwargs["chunk_type"] is None:
        if kwargs["total_chunks"] > 1:
            kwargs["chunk_type"] = "equal_time"
        else:
            kwargs["chunk_type"] = "none"

    if kwargs["processes"] is None:
        kwargs["processes"] = 1

    if kwargs["debugger"] is not None:
        import mozdebug
        if kwargs["debugger"] == "__default__":
            kwargs["debugger"] = mozdebug.get_default_debugger_name()
        debug_info = mozdebug.get_debugger_info(kwargs["debugger"],
                                                kwargs["debugger_args"])
        if debug_info and debug_info.interactive:
            if kwargs["processes"] != 1:
                kwargs["processes"] = 1
            kwargs["no_capture_stdio"] = True
        kwargs["debug_info"] = debug_info
    else:
        kwargs["debug_info"] = None

    if kwargs["binary"] is not None:
        if not os.path.exists(kwargs["binary"]):
            print >> sys.stderr, "Binary path %s does not exist" % kwargs["binary"]
            sys.exit(1)

    if kwargs["ssl_type"] is None:
        if None not in (kwargs["ca_cert_path"], kwargs["host_cert_path"], kwargs["host_key_path"]):
            kwargs["ssl_type"] = "pregenerated"
        elif exe_path(kwargs["openssl_binary"]) is not None:
            kwargs["ssl_type"] = "openssl"
        else:
            kwargs["ssl_type"] = "none"

    if kwargs["ssl_type"] == "pregenerated":
        require_arg(kwargs, "ca_cert_path", lambda x:os.path.exists(x))
        require_arg(kwargs, "host_cert_path", lambda x:os.path.exists(x))
        require_arg(kwargs, "host_key_path", lambda x:os.path.exists(x))

    elif kwargs["ssl_type"] == "openssl":
        path = exe_path(kwargs["openssl_binary"])
        if path is None:
            print >> sys.stderr, "openssl-binary argument missing or not a valid executable"
            sys.exit(1)
        kwargs["openssl_binary"] = path

    if kwargs["ssl_type"] != "none" and kwargs["product"] == "firefox":
        path = exe_path(kwargs["certutil_binary"])
        if path is None:
            print >> sys.stderr, "certutil-binary argument missing or not a valid executable"
            sys.exit(1)
        kwargs["certutil_binary"] = path

    return kwargs


def check_args_update(kwargs):
    set_from_config(kwargs)

    if kwargs["product"] is None:
        kwargs["product"] = "firefox"


def create_parser_update(product_choices=None):
    from mozlog.structured import commandline

    import products

    if product_choices is None:
        config_data = config.load()
        product_choices = products.products_enabled(config_data)

    parser = argparse.ArgumentParser("web-platform-tests-update",
                                     description="Update script for web-platform-tests tests.")
    parser.add_argument("--product", action="store", choices=product_choices,
                        default=None, help="Browser for which metadata is being updated")
    parser.add_argument("--config", action="store", type=abs_path, help="Path to config file")
    parser.add_argument("--metadata", action="store", type=abs_path, dest="metadata_root",
                        help="Path to the folder containing test metadata"),
    parser.add_argument("--tests", action="store", type=abs_path, dest="tests_root",
                        help="Path to web-platform-tests"),
    parser.add_argument("--sync-path", action="store", type=abs_path,
                        help="Path to store git checkout of web-platform-tests during update"),
    parser.add_argument("--remote_url", action="store",
                        help="URL of web-platfrom-tests repository to sync against"),
    parser.add_argument("--branch", action="store", type=abs_path,
                        help="Remote branch to sync against")
    parser.add_argument("--rev", action="store", help="Revision to sync to")
    parser.add_argument("--no-patch", action="store_true",
                        help="Don't create an mq patch or git commit containing the changes.")
    parser.add_argument("--sync", dest="sync", action="store_true", default=False,
                        help="Sync the tests with the latest from upstream")
    parser.add_argument("--ignore-existing", action="store_true", help="When updating test results only consider results from the logfiles provided, not existing expectations.")
    parser.add_argument("--continue", action="store_true", help="Continue a previously started run of the update script")
    parser.add_argument("--abort", action="store_true", help="Clear state from a previous incomplete run of the update script")
    # Should make this required iff run=logfile
    parser.add_argument("run_log", nargs="*", type=abs_path,
                        help="Log file from run of tests")
    commandline.add_logging_group(parser)
    return parser


def create_parser_reduce(product_choices=None):
    parser = create_parser(product_choices)
    parser.add_argument("target", action="store", help="Test id that is unstable")
    return parser


def parse_args():
    parser = create_parser()
    rv = vars(parser.parse_args())
    check_args(rv)
    return rv


def parse_args_update():
    parser = create_parser_update()
    rv = vars(parser.parse_args())
    check_args_update(rv)
    return rv


def parse_args_reduce():
    parser = create_parser_reduce()
    rv = vars(parser.parse_args())
    check_args(rv)
    return rv
