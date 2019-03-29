from __future__ import print_function
import argparse
import os
import sys
from collections import OrderedDict
from distutils.spawn import find_executable
from datetime import timedelta

import config
import wpttest
from formatters import chromium, wptreport, wptscreenshot

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

    if name not in kwargs or not value_func(kwargs[name]):
        print("Missing required argument %s" % name, file=sys.stderr)
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
    parser.add_argument("--manifest-update", action="store_true", default=None,
                        help="Regenerate the test manifest.")
    parser.add_argument("--no-manifest-update", action="store_false", dest="manifest_update",
                        help="Prevent regeneration of the test manifest.")
    parser.add_argument("--manifest-download", action="store_true", default=None,
                        help="Attempt to download a preexisting manifest when updating.")

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
    parser.add_argument("--no-fail-on-unexpected", action="store_false",
                        default=True,
                        dest="fail_on_unexpected",
                        help="Exit with status code 0 when test expectations are violated")

    mode_group = parser.add_argument_group("Mode")
    mode_group.add_argument("--list-test-groups", action="store_true",
                            default=False,
                            help="List the top level directories containing tests that will run.")
    mode_group.add_argument("--list-disabled", action="store_true",
                            default=False,
                            help="List the tests that are disabled on the current platform")
    mode_group.add_argument("--list-tests", action="store_true",
                            default=False,
                            help="List all tests that will run")
    stability_group = mode_group.add_mutually_exclusive_group()
    stability_group.add_argument("--verify", action="store_true",
                                 default=False,
                                 help="Run a stability check on the selected tests")
    stability_group.add_argument("--stability", action="store_true",
                                 default=False,
                                 help=argparse.SUPPRESS)
    mode_group.add_argument("--verify-log-full", action="store_true",
                            default=False,
                            help="Output per-iteration test results when running verify")
    mode_group.add_argument("--verify-repeat-loop", action="store",
                            default=10,
                            help="Number of iterations for a run that reloads each test without restart.",
                            type=int)
    mode_group.add_argument("--verify-repeat-restart", action="store",
                            default=5,
                            help="Number of iterations, for a run that restarts the runner between each iteration",
                            type=int)
    chaos_mode_group = mode_group.add_mutually_exclusive_group()
    chaos_mode_group.add_argument("--verify-no-chaos-mode", action="store_false",
                                  default=True,
                                  dest="verify_chaos_mode",
                                  help="Disable chaos mode when running on Firefox")
    chaos_mode_group.add_argument("--verify-chaos-mode", action="store_true",
                                  default=True,
                                  dest="verify_chaos_mode",
                                  help="Enable chaos mode when running on Firefox")
    mode_group.add_argument("--verify-max-time", action="store",
                            default=None,
                            help="The maximum number of minutes for the job to run",
                            type=lambda x: timedelta(minutes=float(x)))
    output_results_group = mode_group.add_mutually_exclusive_group()
    output_results_group.add_argument("--verify-no-output-results", action="store_false",
                                      dest="verify_output_results",
                                      default=True,
                                      help="Prints individuals test results and messages")
    output_results_group.add_argument("--verify-output-results", action="store_true",
                                      dest="verify_output_results",
                                      default=True,
                                      help="Disable printing individuals test results and messages")

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
    test_selection_group.add_argument("--skip-timeout", action="store_true",
                                      help="Skip tests that are expected to time out")
    test_selection_group.add_argument("--tag", action="append", dest="tags",
                                      help="Labels applied to tests to include in the run. "
                                           "Labels starting dir: are equivalent to top-level directories.")
    test_selection_group.add_argument("--default-exclude", action="store_true",
                                      default=False,
                                      help="Only run the tests explicitly given in arguments. "
                                           "No tests will run if the list is empty, and the "
                                           "program will exit with status code 0.")

    debugging_group = parser.add_argument_group("Debugging")
    debugging_group.add_argument('--debugger', const="__default__", nargs="?",
                                 help="run under a debugger, e.g. gdb or valgrind")
    debugging_group.add_argument('--debugger-args', help="arguments to the debugger")
    debugging_group.add_argument("--rerun", action="store", type=int, default=1,
                                 help="Number of times to re run each test without restarts")
    debugging_group.add_argument("--repeat", action="store", type=int, default=1,
                                 help="Number of times to run the tests, restarting between each run")
    debugging_group.add_argument("--repeat-until-unexpected", action="store_true", default=None,
                                 help="Run tests in a loop until one returns an unexpected result")
    debugging_group.add_argument('--pause-after-test', action="store_true", default=None,
                                 help="Halt the test runner after each test (this happens by default if only a single test is run)")
    debugging_group.add_argument('--no-pause-after-test', dest="pause_after_test", action="store_false",
                                 help="Don't halt the test runner irrespective of the number of tests run")

    debugging_group.add_argument('--pause-on-unexpected', action="store_true",
                                 help="Halt the test runner when an unexpected result is encountered")
    debugging_group.add_argument('--no-restart-on-unexpected', dest="restart_on_unexpected",
                                 default=True, action="store_false",
                                 help="Don't restart on an unexpected result")

    debugging_group.add_argument("--symbols-path", action="store", type=url_or_path,
                                 help="Path or url to symbols file used to analyse crash minidumps.")
    debugging_group.add_argument("--stackwalk-binary", action="store", type=abs_path,
                                 help="Path to stackwalker program used to analyse minidumps.")

    debugging_group.add_argument("--pdb", action="store_true",
                                 help="Drop into pdb on python exception")

    config_group = parser.add_argument_group("Configuration")
    config_group.add_argument("--binary", action="store",
                              type=abs_path, help="Desktop binary to run tests against")
    config_group.add_argument('--binary-arg',
                              default=[], action="append", dest="binary_args",
                              help="Extra argument for the binary")
    config_group.add_argument("--webdriver-binary", action="store", metavar="BINARY",
                              type=abs_path, help="WebDriver server binary to use")
    config_group.add_argument('--webdriver-arg',
                              default=[], action="append", dest="webdriver_args",
                              help="Extra argument for the WebDriver binary")
    config_group.add_argument("--package-name", action="store",
                              help="Android package name to run tests against")
    config_group.add_argument("--device-serial", action="store",
                              help="Running Android instance to connect to, if not emulator-5554")
    config_group.add_argument("--metadata", action="store", type=abs_path, dest="metadata_root",
                              help="Path to root directory containing test metadata"),
    config_group.add_argument("--tests", action="store", type=abs_path, dest="tests_root",
                              help="Path to root directory containing test files"),
    config_group.add_argument("--manifest", action="store", type=abs_path, dest="manifest_path",
                              help="Path to test manifest (default is ${metadata_root}/MANIFEST.json)")
    config_group.add_argument("--run-info", action="store", type=abs_path,
                              help="Path to directory containing extra json files to add to run info")
    config_group.add_argument("--product", action="store", choices=product_choices,
                              default=None, help="Browser against which to run tests")
    config_group.add_argument("--browser-version", action="store",
                              default=None, help="Informative string detailing the browser "
                              "release version. This is included in the run_info data.")
    config_group.add_argument("--browser-channel", action="store",
                              default=None, help="Informative string detailing the browser "
                              "release channel. This is included in the run_info data.")
    config_group.add_argument("--config", action="store", type=abs_path, dest="config",
                              help="Path to config file")
    config_group.add_argument("--install-fonts", action="store_true",
                              default=None,
                              help="Allow the wptrunner to install fonts on your system")
    config_group.add_argument("--font-dir", action="store", type=abs_path, dest="font_dir",
                              help="Path to local font installation directory", default=None)
    config_group.add_argument("--headless", action="store_true",
                              help="Run browser in headless mode", default=None)
    config_group.add_argument("--no-headless", action="store_false", dest="headless",
                              help="Don't run browser in headless mode")

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
    chunking_group.add_argument("--chunk-type", action="store", choices=["none", "hash", "dir_hash"],
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
    gecko_group.add_argument("--stackfix-dir", dest="stackfix_dir", action="store",
                             help="Path to directory containing assertion stack fixing scripts")
    gecko_group.add_argument("--lsan-dir", dest="lsan_dir", action="store",
                             help="Path to directory containing LSAN suppressions file")
    gecko_group.add_argument("--setpref", dest="extra_prefs", action='append',
                             default=[], metavar="PREF=VALUE",
                             help="Defines an extra user preference (overrides those in prefs_root)")
    gecko_group.add_argument("--leak-check", dest="leak_check", action="store_true", default=None,
                             help="Enable leak checking (enabled by default for debug builds, "
                             "silently ignored for opt)")
    gecko_group.add_argument("--no-leak-check", dest="leak_check", action="store_false", default=None,
                             help="Disable leak checking")
    gecko_group.add_argument("--stylo-threads", action="store", type=int, default=1,
                             help="Number of parallel threads to use for stylo")
    gecko_group.add_argument("--reftest-internal", dest="reftest_internal", action="store_true",
                             default=None, help="Enable reftest runner implemented inside Marionette")
    gecko_group.add_argument("--reftest-external", dest="reftest_internal", action="store_false",
                             help="Disable reftest runner implemented inside Marionette")
    gecko_group.add_argument("--reftest-screenshot", dest="reftest_screenshot", action="store",
                             choices=["always", "fail", "unexpected"], default=None,
                             help="With --reftest-internal, when to take a screenshot")
    gecko_group.add_argument("--chaos", dest="chaos_mode_flags", action="store",
                             nargs="?", const=0xFFFFFFFF, type=int,
                             help="Enable chaos mode with the specified feature flag "
                             "(see http://searchfox.org/mozilla-central/source/mfbt/ChaosMode.h for "
                             "details). If no value is supplied, all features are activated")

    servo_group = parser.add_argument_group("Servo-specific")
    servo_group.add_argument("--user-stylesheet",
                             default=[], action="append", dest="user_stylesheets",
                             help="Inject a user CSS stylesheet into every test.")

    sauce_group = parser.add_argument_group("Sauce Labs-specific")
    sauce_group.add_argument("--sauce-browser", dest="sauce_browser",
                             help="Sauce Labs browser name")
    sauce_group.add_argument("--sauce-platform", dest="sauce_platform",
                             help="Sauce Labs OS platform")
    sauce_group.add_argument("--sauce-version", dest="sauce_version",
                             help="Sauce Labs browser version")
    sauce_group.add_argument("--sauce-build", dest="sauce_build",
                             help="Sauce Labs build identifier")
    sauce_group.add_argument("--sauce-tags", dest="sauce_tags", nargs="*",
                             help="Sauce Labs identifying tag", default=[])
    sauce_group.add_argument("--sauce-tunnel-id", dest="sauce_tunnel_id",
                             help="Sauce Connect tunnel identifier")
    sauce_group.add_argument("--sauce-user", dest="sauce_user",
                             help="Sauce Labs user name")
    sauce_group.add_argument("--sauce-key", dest="sauce_key",
                             default=os.environ.get("SAUCE_ACCESS_KEY"),
                             help="Sauce Labs access key")
    sauce_group.add_argument("--sauce-connect-binary",
                             dest="sauce_connect_binary",
                             help="Path to Sauce Connect binary")
    sauce_group.add_argument("--sauce-init-timeout", action="store",
                             type=int, default=30,
                             help="Number of seconds to wait for Sauce "
                                  "Connect tunnel to be available before "
                                  "aborting")
    sauce_group.add_argument("--sauce-connect-arg", action="append",
                             default=[], dest="sauce_connect_args",
                             help="Command-line argument to forward to the "
                                  "Sauce Connect binary (repeatable)")

    webkit_group = parser.add_argument_group("WebKit-specific")
    webkit_group.add_argument("--webkit-port", dest="webkit_port",
                             help="WebKit port")

    parser.add_argument("test_list", nargs="*",
                        help="List of URLs for tests to run, or paths including tests to run. "
                             "(equivalent to --include)")

    def screenshot_api_wrapper(formatter, api):
        formatter.api = api
        return formatter

    commandline.fmt_options["api"] = (screenshot_api_wrapper,
                                      "Cache API (default: %s)" % wptscreenshot.DEFAULT_API,
                                      {"wptscreenshot"}, "store")

    commandline.log_formatters["chromium"] = (chromium.ChromiumFormatter, "Chromium Layout Tests format")
    commandline.log_formatters["wptreport"] = (wptreport.WptreportFormatter, "wptreport format")
    commandline.log_formatters["wptscreenshot"] = (wptscreenshot.WptscreenshotFormatter, "wpt.fyi screenshots")

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

    if kwargs.get("manifest_path"):
        if "/" not in kwargs["test_paths"]:
            kwargs["test_paths"]["/"] = {}
        kwargs["test_paths"]["/"]["manifest_path"] = kwargs["manifest_path"]

    kwargs["suite_name"] = kwargs["config"].get("web-platform-tests", {}).get("name", "web-platform-tests")


    check_paths(kwargs)


def get_test_paths(config):
    # Set up test_paths
    test_paths = OrderedDict()

    for section in config.iterkeys():
        if section.startswith("manifest:"):
            manifest_opts = config.get(section)
            url_base = manifest_opts.get("url_base", "/")
            test_paths[url_base] = {
                "tests_path": manifest_opts.get_path("tests"),
                "metadata_path": manifest_opts.get_path("metadata"),
            }
            if "manifest" in manifest_opts:
                test_paths[url_base]["manifest_path"] = manifest_opts.get_path("manifest")

    return test_paths


def exe_path(name):
    if name is None:
        return

    path = find_executable(name)
    if path and os.access(path, os.X_OK):
        return path
    else:
        return None


def check_paths(kwargs):
    for test_paths in kwargs["test_paths"].itervalues():
        if not ("tests_path" in test_paths and
                "metadata_path" in test_paths):
            print("Fatal: must specify both a test path and metadata path")
            sys.exit(1)
        if "manifest_path" not in test_paths:
            test_paths["manifest_path"] = os.path.join(test_paths["metadata_path"],
                                                       "MANIFEST.json")
        for key, path in test_paths.iteritems():
            name = key.split("_", 1)[0]

            if name == "manifest":
                # For the manifest we can create it later, so just check the path
                # actually exists
                path = os.path.dirname(path)

            if not os.path.exists(path):
                print("Fatal: %s path %s does not exist" % (name, path))
                sys.exit(1)

            if not os.path.isdir(path):
                print("Fatal: %s path %s is not a directory" % (name, path))
                sys.exit(1)


def check_args(kwargs):
    set_from_config(kwargs)

    if kwargs["product"] is None:
        kwargs["product"] = "firefox"

    if kwargs["manifest_update"] is None:
        kwargs["manifest_update"] = True

    if "sauce" in kwargs["product"]:
        kwargs["pause_after_test"] = False

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
            kwargs["chunk_type"] = "dir_hash"
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
            print("Binary path %s does not exist" % kwargs["binary"], file=sys.stderr)
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
            print("openssl-binary argument missing or not a valid executable", file=sys.stderr)
            sys.exit(1)
        kwargs["openssl_binary"] = path

    if kwargs["ssl_type"] != "none" and kwargs["product"] == "firefox" and kwargs["certutil_binary"]:
        path = exe_path(kwargs["certutil_binary"])
        if path is None:
            print("certutil-binary argument missing or not a valid executable", file=sys.stderr)
            sys.exit(1)
        kwargs["certutil_binary"] = path

    if kwargs['extra_prefs']:
        # If a single pref is passed in as a string, make it a list
        if type(kwargs['extra_prefs']) in (str, unicode):
            kwargs['extra_prefs'] = [kwargs['extra_prefs']]
        missing = any('=' not in prefarg for prefarg in kwargs['extra_prefs'])
        if missing:
            print("Preferences via --setpref must be in key=value format", file=sys.stderr)
            sys.exit(1)
        kwargs['extra_prefs'] = [tuple(prefarg.split('=', 1)) for prefarg in
                                 kwargs['extra_prefs']]

    if kwargs["reftest_internal"] is None:
        kwargs["reftest_internal"] = True

    if kwargs["lsan_dir"] is None:
        kwargs["lsan_dir"] = kwargs["prefs_root"]

    if kwargs["reftest_screenshot"] is None:
        kwargs["reftest_screenshot"] = "unexpected"

    return kwargs


def check_args_update(kwargs):
    set_from_config(kwargs)

    if kwargs["product"] is None:
        kwargs["product"] = "firefox"
    if kwargs["patch"] is None:
        kwargs["patch"] = kwargs["sync"]

    for item in kwargs["run_log"]:
        if os.path.isdir(item):
            print("Log file %s is a directory" % item, file=sys.stderr)
            sys.exit(1)

    return kwargs


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
    parser.add_argument("--manifest", action="store", type=abs_path, dest="manifest_path",
                        help="Path to test manifest (default is ${metadata_root}/MANIFEST.json)")
    parser.add_argument("--sync-path", action="store", type=abs_path,
                        help="Path to store git checkout of web-platform-tests during update"),
    parser.add_argument("--remote_url", action="store",
                        help="URL of web-platfrom-tests repository to sync against"),
    parser.add_argument("--branch", action="store", type=abs_path,
                        help="Remote branch to sync against")
    parser.add_argument("--rev", action="store", help="Revision to sync to")
    parser.add_argument("--patch", action="store_true", dest="patch", default=None,
                        help="Create a VCS commit containing the changes.")
    parser.add_argument("--no-patch", action="store_false", dest="patch",
                        help="Don't create a VCS commit containing the changes.")
    parser.add_argument("--sync", dest="sync", action="store_true", default=False,
                        help="Sync the tests with the latest from upstream (implies --patch)")
    parser.add_argument("--ignore-existing", action="store_true",
                        help="When updating test results only consider results from the logfiles provided, not existing expectations.")
    parser.add_argument("--stability", nargs="?", action="store", const="unstable", default=None,
        help=("Reason for disabling tests. When updating test results, disable tests that have "
              "inconsistent results across many runs with the given reason."))
    parser.add_argument("--no-remove-obsolete", action="store_false", dest="remove_obsolete", default=True,
                        help=("Don't remove metadata files that no longer correspond to a test file"))
    parser.add_argument("--no-store-state", action="store_false", dest="store_state",
                        help="Store state so that steps can be resumed after failure")
    parser.add_argument("--continue", action="store_true",
                        help="Continue a previously started run of the update script")
    parser.add_argument("--abort", action="store_true",
                        help="Clear state from a previous incomplete run of the update script")
    parser.add_argument("--exclude", action="store", nargs="*",
                        help="List of glob-style paths to exclude when syncing tests")
    parser.add_argument("--include", action="store", nargs="*",
                        help="List of glob-style paths to include which would otherwise be excluded when syncing tests")
    parser.add_argument("--extra-property", action="append", default=[],
                        help="Extra property from run_info.json to use in metadata update")
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
