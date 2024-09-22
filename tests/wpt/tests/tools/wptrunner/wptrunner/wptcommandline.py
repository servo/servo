# mypy: allow-untyped-defs

import argparse
import os
import sys
from collections import OrderedDict
from shutil import which
from datetime import timedelta
from typing import Mapping, Optional

from . import config
from . import products
from . import wpttest
from .formatters import chromium, wptreport, wptscreenshot


def abs_path(path):
    return os.path.abspath(os.path.expanduser(path))


def url_or_path(path):
    from urllib.parse import urlparse

    parsed = urlparse(path)
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

    if product_choices is None:
        product_choices = products.product_list

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
    parser.add_argument("--no-manifest-download", action="store_false", dest="manifest_download",
                        help="Prevent download of the test manifest.")

    parser.add_argument("--timeout-multiplier", action="store", type=float, default=None,
                        help="Multiplier relative to standard test timeout to use")
    parser.add_argument("--run-by-dir", type=int, nargs="?", default=False,
                        help="Split run into groups by directories. With a parameter,"
                        "limit the depth of splits e.g. --run-by-dir=1 to split by top-level"
                        "directory")
    parser.add_argument("-f", "--fully-parallel", action='store_true',
                        help='Run every test in a separate group for fully parallelism.')
    parser.add_argument("--processes", action="store", type=int, default=None,
                        help="Number of simultaneous processes to use")
    parser.add_argument("--max-restarts", action="store", type=int, default=5,
                        help="Maximum number of browser restart retries")

    parser.add_argument("--no-capture-stdio", action="store_true", default=False,
                        help="Don't capture stdio and write to logging")
    parser.add_argument("--no-fail-on-unexpected", action="store_false",
                        default=True,
                        dest="fail_on_unexpected",
                        help="Exit with status code 0 when test expectations are violated")
    parser.add_argument("--no-fail-on-unexpected-pass", action="store_false",
                        default=True,
                        dest="fail_on_unexpected_pass",
                        help="Exit with status code 0 when all unexpected results are PASS")
    parser.add_argument("--no-restart-on-new-group", action="store_false",
                        default=True,
                        dest="restart_on_new_group",
                        help="Don't restart test runner when start a new test group")

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
    mode_group.add_argument("--repeat-max-time", action="store",
                            default=100,
                            help="The maximum number of minutes for the test suite to attempt repeat runs",
                            type=int)
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
    test_selection_group.add_argument("--subsuite-file", action="store",
                                      help="Path to JSON file containing subsuite configuration")
    # TODO use an empty string argument for the default subsuite
    test_selection_group.add_argument("--subsuite", action="append", dest="subsuites",
                                      help="Subsuite names to run. Runs all subsuites when omitted.")
    test_selection_group.add_argument("--small-subsuite-size", default=50, type=int,
                                      help="Maximum number of tests a subsuite can have to be treated as small subsuite."
                                      "Tests from a small subsuite will be grouped in one group.")
    test_selection_group.add_argument("--include", action="append",
                                      help="URL prefix to include")
    test_selection_group.add_argument("--include-file", action="store",
                                      help="A file listing URL prefix for tests")
    test_selection_group.add_argument("--exclude", action="append",
                                      help="URL prefix to exclude")
    test_selection_group.add_argument("--include-manifest", type=abs_path,
                                      help="Path to manifest listing tests to include")
    test_selection_group.add_argument("--test-groups", dest="test_groups_file", type=abs_path,
                                      help="Path to json file containing a mapping {group_name: [test_ids]}")
    test_selection_group.add_argument("--skip-timeout", action="store_true",
                                      help="Skip tests that are expected to time out")
    test_selection_group.add_argument("--skip-crash", action="store_true",
                                      help="Skip tests that are expected to crash")
    test_selection_group.add_argument("--skip-implementation-status",
                                      action="append",
                                      choices=["not-implementing", "backlog", "implementing"],
                                      help="Skip tests that have the given implementation status")
    # TODO(bashi): Remove this when WebTransport over HTTP/3 server is enabled by default.
    test_selection_group.add_argument("--enable-webtransport-h3",
                                      action="store_true",
                                      dest="enable_webtransport_h3",
                                      default=None,
                                      help="Enable tests that require WebTransport over HTTP/3 server (default: false)")
    test_selection_group.add_argument("--no-enable-webtransport-h3", action="store_false", dest="enable_webtransport_h3",
                                      help="Do not enable WebTransport tests on experimental channels")
    test_selection_group.add_argument("--tag", action="append", dest="tags",
                                      help="Labels applied to tests to include in the run. "
                                           "Labels starting dir: are equivalent to top-level directories.")
    test_selection_group.add_argument("--exclude-tag", action="append", dest="exclude_tags",
                                      help="Labels applied to tests to exclude in the run. Takes precedence over `--tag`. "
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
    debugging_group.add_argument('--retry-unexpected', type=int, default=0,
                                 help=('Maximum number of times to retry unexpected tests. '
                                       'A test is retried until it gets one of the expected status, '
                                       'or until it exhausts the maximum number of retries.'))
    debugging_group.add_argument('--pause-after-test', action="store_true", default=None,
                                 help="Halt the test runner after each test (this happens by default if only a single test is run)")
    debugging_group.add_argument('--no-pause-after-test', dest="pause_after_test", action="store_false",
                                 help="Don't halt the test runner irrespective of the number of tests run")
    debugging_group.add_argument('--debug-test', dest="debug_test", action="store_true",
                                 help="Run tests with additional debugging features enabled")

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
    debugging_group.add_argument("--leak-check", dest="leak_check", action="store_true", default=None,
                                 help=("Enable leak checking for supported browsers "
                                       "(Gecko: enabled by default for debug builds, "
                                       "silently ignored for opt, mobile)"))
    debugging_group.add_argument("--no-leak-check", dest="leak_check", action="store_false", default=None,
                                 help="Disable leak checking")

    android_group = parser.add_argument_group("Android specific arguments")
    android_group.add_argument("--adb-binary", action="store",
                        help="Path to adb binary to use")
    android_group.add_argument("--package-name", action="store",
                        help="Android package name to run tests against")
    android_group.add_argument("--keep-app-data-directory", action="store_true",
                        help="Don't delete the app data directory")
    android_group.add_argument("--device-serial", action="append", default=[],
                        help="Running Android instances to connect to, if not emulator-5554")

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
                              help="Install additional system fonts on your system")
    config_group.add_argument("--no-install-fonts", dest="install_fonts", action="store_false",
                              help="Do not install additional system fonts on your system")
    config_group.add_argument("--font-dir", action="store", type=abs_path, dest="font_dir",
                              help="Path to local font installation directory", default=None)
    config_group.add_argument("--inject-script", action="store", dest="inject_script", default=None,
                              help="Path to script file to inject, useful for testing polyfills.")
    config_group.add_argument("--headless", action="store_true",
                              help="Run browser in headless mode", default=None)
    config_group.add_argument("--no-headless", action="store_false", dest="headless",
                              help="Don't run browser in headless mode")
    config_group.add_argument("--instrument-to-file", action="store",
                              help="Path to write instrumentation logs to")
    config_group.add_argument("--suppress-handler-traceback", action="store_true", default=None,
                              help="Don't write the stacktrace for exceptions in server handlers")
    config_group.add_argument("--no-suppress-handler-traceback", action="store_false",
                              dest="supress_handler_traceback",
                              help="Write the stacktrace for exceptions in server handlers")

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
    chunking_group.add_argument("--chunk-type", action="store",
                                choices=["none", "hash", "id_hash", "dir_hash"],
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
    gecko_group.add_argument("--preload-browser", dest="preload_browser", action="store_true",
                             default=None, help="Preload a gecko instance for faster restarts")
    gecko_group.add_argument("--no-preload-browser", dest="preload_browser", action="store_false",
                             default=None, help="Don't preload a gecko instance for faster restarts")
    gecko_group.add_argument("--disable-e10s", dest="gecko_e10s", action="store_false", default=True,
                             help="Run tests without electrolysis preferences")
    gecko_group.add_argument("--disable-fission", dest="disable_fission", action="store_true", default=False,
                             help="Disable fission in Gecko.")
    gecko_group.add_argument("--stackfix-dir", dest="stackfix_dir", action="store",
                             help="Path to directory containing assertion stack fixing scripts")
    gecko_group.add_argument("--specialpowers-path", action="store",
                             help="Path to specialPowers extension xpi file")
    gecko_group.add_argument("--setpref", dest="extra_prefs", action='append',
                             default=[], metavar="PREF=VALUE",
                             help="Defines an extra user preference (overrides those in prefs_root)")
    gecko_group.add_argument("--reftest-internal", dest="reftest_internal", action="store_true",
                             default=None, help="Enable reftest runner implemented inside Marionette")
    gecko_group.add_argument("--reftest-external", dest="reftest_internal", action="store_false",
                             help="Disable reftest runner implemented inside Marionette")
    gecko_group.add_argument("--reftest-screenshot", dest="reftest_screenshot", action="store",
                             choices=["always", "fail", "unexpected"], default=None,
                             help="With --reftest-internal, when to take a screenshot")
    gecko_group.add_argument("--chaos", dest="chaos_mode_flags", action="store",
                             nargs="?", const=0xFFFFFFFF, type=lambda x: int(x, 16),
                             help="Enable chaos mode with the specified feature flag "
                             "(see http://searchfox.org/mozilla-central/source/mfbt/ChaosMode.h for "
                             "details). If no value is supplied, all features are activated")

    gecko_view_group = parser.add_argument_group("GeckoView-specific")
    gecko_view_group.add_argument("--setenv", dest="env", action="append", default=[],
                                  help="Set target environment variable, like FOO=BAR")

    servo_group = parser.add_argument_group("Servo-specific")
    servo_group.add_argument("--user-stylesheet",
                             default=[], action="append", dest="user_stylesheets",
                             help="Inject a user CSS stylesheet into every test.")

    chrome_group = parser.add_argument_group("Chrome-specific")
    chrome_group.add_argument("--enable-mojojs", action="store_true", default=False,
                             help="Enable MojoJS for testing. Note that this flag is usally "
                             "enabled automatically by `wpt run`, if it succeeds in downloading "
                             "the right version of mojojs.zip or if --mojojs-path is specified.")
    chrome_group.add_argument("--mojojs-path",
                             help="Path to mojojs gen/ directory. If it is not specified, `wpt run` "
                             "will download and extract mojojs.zip into _venv2/mojojs/gen.")
    chrome_group.add_argument("--enable-swiftshader", action="store_true", default=False,
                             help="Enable SwiftShader for CPU-based 3D graphics. This can be used "
                             "in environments with no hardware GPU available.")
    chrome_group.add_argument("--enable-experimental", action="store_true", dest="enable_experimental",
                              help="Enable --enable-experimental-web-platform-features flag", default=None)
    chrome_group.add_argument("--no-enable-experimental", action="store_false", dest="enable_experimental",
                              help="Do not enable --enable-experimental-web-platform-features flag "
                              "on experimental channels")
    chrome_group.add_argument(
        "--enable-sanitizer",
        action="store_true",
        dest="sanitizer_enabled",
        help="Only alert on sanitizer-related errors and crashes.")
    chrome_group.add_argument(
        "--reuse-window",
        action="store_true",
        help=("Reuse a window across `testharness.js` tests where possible, "
              "which can speed up testing. Also useful for ensuring that the "
              "renderer process has a stable PID for a debugger to attach to."))

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

    taskcluster_group = parser.add_argument_group("Taskcluster-specific")
    taskcluster_group.add_argument("--github-checks-text-file",
                                   type=str,
                                   help="Path to GitHub checks output file")

    webkit_group = parser.add_argument_group("WebKit-specific")
    webkit_group.add_argument("--webkit-port", dest="webkit_port",
                              help="WebKit port")

    safari_group = parser.add_argument_group("Safari-specific")
    safari_group.add_argument("--kill-safari", dest="kill_safari", action="store_true", default=False,
                              help="Kill Safari when stopping the browser")

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

    kwargs["product"] = products.Product(kwargs["config"], kwargs["product"])

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

    for section, values in keys.items():
        for config_value, kw_value, is_path in values:
            if kw_value in kwargs and kwargs[kw_value] is None:
                if not is_path:
                    new_value = kwargs["config"].get(section, config.ConfigDict({})).get(config_value)
                else:
                    new_value = kwargs["config"].get(section, config.ConfigDict({})).get_path(config_value)
                kwargs[kw_value] = new_value

    test_paths = get_test_paths(kwargs["config"],
                                kwargs["tests_root"],
                                kwargs["metadata_root"],
                                kwargs["manifest_path"])
    check_paths(test_paths)
    kwargs["test_paths"] = test_paths

    kwargs["suite_name"] = kwargs["config"].get("web-platform-tests", {}).get("name", "web-platform-tests")



class TestRoot:
    def __init__(self, tests_path: str, metadata_path: str, manifest_path: Optional[str] = None):
        self.tests_path = tests_path
        self.metadata_path = metadata_path
        if manifest_path is None:
            manifest_path = os.path.join(metadata_path, "MANIFEST.json")

        self.manifest_path = manifest_path


TestPaths = Mapping[str, TestRoot]


def get_test_paths(config: Mapping[str, config.ConfigDict],
                   tests_path_override: Optional[str] = None,
                   metadata_path_override: Optional[str] = None,
                   manifest_path_override: Optional[str] = None) -> TestPaths:
    # Set up test_paths
    test_paths = OrderedDict()

    for section in config.keys():
        if section.startswith("manifest:"):
            manifest_opts = config[section]
            url_base = manifest_opts.get("url_base", "/")
            tests_path = manifest_opts.get_path("tests")
            if tests_path is None:
                raise ValueError(f"Missing `tests` key in configuration with url_base {url_base}")
            metadata_path = manifest_opts.get_path("metadata")
            if metadata_path is None:
                raise ValueError(f"Missing `metadata` key in configuration with url_base {url_base}")
            manifest_path = manifest_opts.get_path("manifest")

            if url_base == "/":
                if tests_path_override is not None:
                    tests_path = tests_path_override
                if metadata_path_override is not None:
                    metadata_path = metadata_path_override
                if manifest_path_override is not None:
                    manifest_path = manifest_path_override

            test_paths[url_base] = TestRoot(tests_path, metadata_path, manifest_path)

    if "/" not in test_paths:
        if tests_path_override is None or metadata_path_override is None:
            raise ValueError("No ini file configures the root url, "
                             "so --tests and --metadata arguments are mandatory")
        test_paths["/"] = TestRoot(tests_path_override,
                                   metadata_path_override,
                                   manifest_path_override)

    return test_paths


def exe_path(name: Optional[str]) -> Optional[str]:
    if name is None:
        return None

    return which(name)


def check_paths(test_paths: TestPaths) -> None:
    for test_root in test_paths.values():
        for key in ["tests_path", "metadata_path", "manifest_path"]:
            name = key.split("_", 1)[0]
            path = getattr(test_root, key)

            if name == "manifest":
                # For the manifest we can create it later, so just check the path
                # actually exists
                path = os.path.dirname(path)

            if not os.path.exists(path):
                print(f"Fatal: {name} path {path} does not exist")
                sys.exit(1)

            if not os.path.isdir(path):
                print(f"Fatal: {name} path {path} is not a directory")
                sys.exit(1)


def check_args(kwargs):
    set_from_config(kwargs)

    if kwargs["manifest_update"] is None:
        kwargs["manifest_update"] = True

    if "sauce" in kwargs["product"].name:
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

    if sum([
        kwargs["test_groups_file"] is not None,
        kwargs["run_by_dir"] is not False,
        kwargs["fully_parallel"],
    ]) > 1:
        print('Must pass up to one of: --test-groups, --run-by-dir, --fully-parallel')
        sys.exit(1)

    if (kwargs["test_groups_file"] is not None and
        not os.path.exists(kwargs["test_groups_file"])):
        print("--test-groups file %s not found" % kwargs["test_groups_file"])
        sys.exit(1)

    # When running on Android, the number of workers is decided by the number of
    # emulators. Each worker will use one emulator to run the Android browser.
    if kwargs["device_serial"]:
        if kwargs["processes"] is None:
            kwargs["processes"] = len(kwargs["device_serial"])
        elif len(kwargs["device_serial"]) != kwargs["processes"]:
            print("--processes does not match number of devices")
            sys.exit(1)
        elif len(set(kwargs["device_serial"])) != len(kwargs["device_serial"]):
            print("Got duplicate --device-serial value")
            sys.exit(1)

    if kwargs["processes"] is None:
        from manifest import mputil  # type: ignore
        kwargs["processes"] = mputil.max_parallelism() if kwargs["fully_parallel"] else 1

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

    if kwargs["ssl_type"] != "none" and kwargs["product"].name == "firefox" and kwargs["certutil_binary"]:
        path = exe_path(kwargs["certutil_binary"])
        if path is None:
            print("certutil-binary argument missing or not a valid executable", file=sys.stderr)
            sys.exit(1)
        kwargs["certutil_binary"] = path

    if kwargs['extra_prefs']:
        missing = any('=' not in prefarg for prefarg in kwargs['extra_prefs'])
        if missing:
            print("Preferences via --setpref must be in key=value format", file=sys.stderr)
            sys.exit(1)
        kwargs['extra_prefs'] = [tuple(prefarg.split('=', 1)) for prefarg in
                                 kwargs['extra_prefs']]

    if kwargs["reftest_internal"] is None:
        kwargs["reftest_internal"] = True

    if kwargs["reftest_screenshot"] is None:
        kwargs["reftest_screenshot"] = "unexpected" if not kwargs["debug_test"] else "always"

    if kwargs["preload_browser"] is None:
        # Default to preloading a gecko instance if we're only running a single process
        kwargs["preload_browser"] = kwargs["processes"] == 1

    if kwargs["tags"] and kwargs["exclude_tags"]:
        contradictory = set(kwargs["tags"]) & set(kwargs["exclude_tags"])
        if contradictory:
            print("contradictory tags found; exclusion will take precedence:", contradictory)

    return kwargs


def check_args_metadata_update(kwargs):
    set_from_config(kwargs)

    for item in kwargs["run_log"]:
        if os.path.isdir(item):
            print("Log file %s is a directory" % item, file=sys.stderr)
            sys.exit(1)

    if kwargs["properties_file"] is None and not kwargs["no_properties_file"]:
        default_file = os.path.join(kwargs["test_paths"]["/"].metadata_path,
                                    "update_properties.json")
        if os.path.exists(default_file):
            kwargs["properties_file"] = default_file

    return kwargs


def check_args_update(kwargs):
    kwargs = check_args_metadata_update(kwargs)

    if kwargs["patch"] is None:
        kwargs["patch"] = kwargs["sync"]

    return kwargs


def create_parser_metadata_update(product_choices=None):
    from mozlog.structured import commandline

    from . import products

    if product_choices is None:
        product_choices = products.product_list

    parser = argparse.ArgumentParser("web-platform-tests-update",
                                     description="Update script for web-platform-tests tests.")
    # This will be removed once all consumers are updated to the properties-file based system
    parser.add_argument("--product", action="store", choices=product_choices,
                        default="firefox", help=argparse.SUPPRESS)
    parser.add_argument("--config", action="store", type=abs_path, help="Path to config file")
    parser.add_argument("--metadata", action="store", type=abs_path, dest="metadata_root",
                        help="Path to the folder containing test metadata"),
    parser.add_argument("--tests", action="store", type=abs_path, dest="tests_root",
                        help="Path to web-platform-tests"),
    parser.add_argument("--manifest", action="store", type=abs_path, dest="manifest_path",
                        help="Path to test manifest (default is ${metadata_root}/MANIFEST.json)")
    parser.add_argument("--full", action="store_true", default=False,
                        help="For all tests that are updated, remove any existing conditions and missing subtests")
    parser.add_argument("--disable-intermittent", nargs="?", action="store", const="unstable", default=None,
        help=("Reason for disabling tests. When updating test results, disable tests that have "
              "inconsistent results across many runs with the given reason."))
    parser.add_argument("--update-intermittent", action="store_true", default=False,
                        help="Update test metadata with expected intermittent statuses.")
    parser.add_argument("--remove-intermittent", action="store_true", default=False,
                        help="Remove obsolete intermittent statuses from expected statuses.")
    parser.add_argument("--no-remove-obsolete", action="store_false", dest="remove_obsolete", default=True,
                        help="Don't remove metadata files that no longer correspond to a test file")
    parser.add_argument("--properties-file",
                        help="""Path to a JSON file containing run_info properties to use in update. This must be of the form
                        {"properties": [<name>], "dependents": {<property name>: [<name>]}}""")
    parser.add_argument("--no-properties-file", action="store_true",
                        help="Don't use the default properties file at "
                        "${metadata_root}/update_properties.json, even if it exists.")
    parser.add_argument("--extra-property", action="append", default=[],
                        help="Extra property from run_info.json to use in metadata update.")
    # TODO: Should make this required iff run=logfile
    parser.add_argument("run_log", nargs="*", type=abs_path,
                        help="Log file from run of tests")
    commandline.add_logging_group(parser)
    return parser


def create_parser_update(product_choices=None):
    parser = create_parser_metadata_update(product_choices)
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
