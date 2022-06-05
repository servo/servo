# mypy: allow-untyped-defs

import json
import os
import sys
from datetime import datetime, timedelta

import wptserve
from wptserve import sslutils

from . import environment as env
from . import instruments
from . import mpcontext
from . import products
from . import testloader
from . import wptcommandline
from . import wptlogging
from . import wpttest
from mozlog import capture, handlers
from .font import FontInstaller
from .testrunner import ManagerGroup

here = os.path.dirname(__file__)

logger = None

"""Runner for web-platform-tests

The runner has several design goals:

* Tests should run with no modification from upstream.

* Tests should be regarded as "untrusted" so that errors, timeouts and even
  crashes in the tests can be handled without failing the entire test run.

* For performance tests can be run in multiple browsers in parallel.

The upstream repository has the facility for creating a test manifest in JSON
format. This manifest is used directly to determine which tests exist. Local
metadata files are used to store the expected test results.
"""

def setup_logging(*args, **kwargs):
    global logger
    logger = wptlogging.setup(*args, **kwargs)
    return logger


def get_loader(test_paths, product, debug=None, run_info_extras=None, chunker_kwargs=None,
               test_groups=None, **kwargs):
    if run_info_extras is None:
        run_info_extras = {}

    run_info = wpttest.get_run_info(kwargs["run_info"], product,
                                    browser_version=kwargs.get("browser_version"),
                                    browser_channel=kwargs.get("browser_channel"),
                                    verify=kwargs.get("verify"),
                                    debug=debug,
                                    extras=run_info_extras,
                                    enable_webrender=kwargs.get("enable_webrender"),
                                    device_serials=kwargs.get("device_serial"),
                                    adb_binary=kwargs.get("adb_binary"))

    test_manifests = testloader.ManifestLoader(test_paths, force_manifest_update=kwargs["manifest_update"],
                                               manifest_download=kwargs["manifest_download"]).load()

    manifest_filters = []

    include = kwargs["include"]
    if kwargs["include_file"]:
        include = include or []
        include.extend(testloader.read_include_from_file(kwargs["include_file"]))
    if test_groups:
        include = testloader.update_include_for_groups(test_groups, include)

    if include or kwargs["exclude"] or kwargs["include_manifest"] or kwargs["default_exclude"]:
        manifest_filters.append(testloader.TestFilter(include=include,
                                                      exclude=kwargs["exclude"],
                                                      manifest_path=kwargs["include_manifest"],
                                                      test_manifests=test_manifests,
                                                      explicit=kwargs["default_exclude"]))

    ssl_enabled = sslutils.get_cls(kwargs["ssl_type"]).ssl_enabled
    h2_enabled = wptserve.utils.http2_compatible()
    test_loader = testloader.TestLoader(test_manifests,
                                        kwargs["test_types"],
                                        run_info,
                                        manifest_filters=manifest_filters,
                                        chunk_type=kwargs["chunk_type"],
                                        total_chunks=kwargs["total_chunks"],
                                        chunk_number=kwargs["this_chunk"],
                                        include_https=ssl_enabled,
                                        include_h2=h2_enabled,
                                        include_webtransport_h3=kwargs["enable_webtransport_h3"],
                                        skip_timeout=kwargs["skip_timeout"],
                                        skip_implementation_status=kwargs["skip_implementation_status"],
                                        chunker_kwargs=chunker_kwargs)
    return run_info, test_loader


def list_test_groups(test_paths, product, **kwargs):
    env.do_delayed_imports(logger, test_paths)

    run_info_extras = products.Product(kwargs["config"], product).run_info_extras(**kwargs)

    run_info, test_loader = get_loader(test_paths, product,
                                       run_info_extras=run_info_extras, **kwargs)

    for item in sorted(test_loader.groups(kwargs["test_types"])):
        print(item)


def list_disabled(test_paths, product, **kwargs):
    env.do_delayed_imports(logger, test_paths)

    rv = []

    run_info_extras = products.Product(kwargs["config"], product).run_info_extras(**kwargs)

    run_info, test_loader = get_loader(test_paths, product,
                                       run_info_extras=run_info_extras, **kwargs)

    for test_type, tests in test_loader.disabled_tests.items():
        for test in tests:
            rv.append({"test": test.id, "reason": test.disabled()})
    print(json.dumps(rv, indent=2))


def list_tests(test_paths, product, **kwargs):
    env.do_delayed_imports(logger, test_paths)

    run_info_extras = products.Product(kwargs["config"], product).run_info_extras(**kwargs)

    run_info, test_loader = get_loader(test_paths, product,
                                       run_info_extras=run_info_extras, **kwargs)

    for test in test_loader.test_ids:
        print(test)


def get_pause_after_test(test_loader, **kwargs):
    if kwargs["pause_after_test"] is None:
        if kwargs["repeat_until_unexpected"]:
            return False
        if kwargs["headless"]:
            return False
        if kwargs["debug_test"]:
            return True
        tests = test_loader.tests
        is_single_testharness = (sum(len(item) for item in tests.values()) == 1 and
                                 len(tests.get("testharness", [])) == 1)
        if kwargs["repeat"] == 1 and kwargs["rerun"] == 1 and is_single_testharness:
            return True
        return False
    return kwargs["pause_after_test"]


def run_test_iteration(test_status, test_loader, test_source_kwargs, test_source_cls, run_info,
                       recording, test_environment, product, run_test_kwargs):
    """Runs the entire test suite.
    This is called for each repeat run requested."""
    tests = []
    for test_type in test_loader.test_types:
        tests.extend(test_loader.tests[test_type])

    try:
        test_groups = test_source_cls.tests_by_group(
            tests, **test_source_kwargs)
    except Exception:
        logger.critical("Loading tests failed")
        return False

    logger.suite_start(test_groups,
                       name='web-platform-test',
                       run_info=run_info,
                       extra={"run_by_dir": run_test_kwargs["run_by_dir"]})
    for test_type in run_test_kwargs["test_types"]:
        logger.info(f"Running {test_type} tests")

        browser_cls = product.get_browser_cls(test_type)

        browser_kwargs = product.get_browser_kwargs(logger,
                                                    test_type,
                                                    run_info,
                                                    config=test_environment.config,
                                                    num_test_groups=len(test_groups),
                                                    **run_test_kwargs)

        executor_cls = product.executor_classes.get(test_type)
        executor_kwargs = product.get_executor_kwargs(logger,
                                                      test_type,
                                                      test_environment,
                                                      run_info,
                                                      **run_test_kwargs)

        if executor_cls is None:
            logger.error(f"Unsupported test type {test_type} for product {product.name}")
            continue

        for test in test_loader.disabled_tests[test_type]:
            logger.test_start(test.id)
            logger.test_end(test.id, status="SKIP")
            test_status.skipped += 1

        if test_type == "testharness":
            run_tests = {"testharness": []}
            for test in test_loader.tests["testharness"]:
                if ((test.testdriver and not executor_cls.supports_testdriver) or
                        (test.jsshell and not executor_cls.supports_jsshell)):
                    logger.test_start(test.id)
                    logger.test_end(test.id, status="SKIP")
                    test_status.skipped += 1
                else:
                    run_tests["testharness"].append(test)
        else:
            run_tests = test_loader.tests

        recording.pause()
        with ManagerGroup("web-platform-tests",
                          run_test_kwargs["processes"],
                          test_source_cls,
                          test_source_kwargs,
                          browser_cls,
                          browser_kwargs,
                          executor_cls,
                          executor_kwargs,
                          run_test_kwargs["rerun"],
                          run_test_kwargs["pause_after_test"],
                          run_test_kwargs["pause_on_unexpected"],
                          run_test_kwargs["restart_on_unexpected"],
                          run_test_kwargs["debug_info"],
                          not run_test_kwargs["no_capture_stdio"],
                          recording=recording) as manager_group:
            try:
                manager_group.run(test_type, run_tests)
            except KeyboardInterrupt:
                logger.critical("Main thread got signal")
                manager_group.stop()
                raise
            test_status.total_tests += manager_group.test_count()
            test_status.unexpected += manager_group.unexpected_count()
            test_status.unexpected_pass += manager_group.unexpected_pass_count()

    return True


def evaluate_runs(test_status, run_test_kwargs):
    """Evaluates the test counts after the given number of repeat runs has finished"""
    if test_status.total_tests == 0:
        if test_status.skipped > 0:
            logger.warning("All requested tests were skipped")
        else:
            if run_test_kwargs["default_exclude"]:
                logger.info("No tests ran")
                return True
            else:
                logger.critical("No tests ran")
                return False

    if test_status.unexpected and not run_test_kwargs["fail_on_unexpected"]:
        logger.info(f"Tolerating {test_status.unexpected} unexpected results")
        return True

    all_unexpected_passed = (test_status.unexpected and
                             test_status.unexpected == test_status.unexpected_pass)
    if all_unexpected_passed and not run_test_kwargs["fail_on_unexpected_pass"]:
        logger.info(f"Tolerating {test_status.unexpected_pass} unexpected results "
                    "because they all PASS")
        return True

    return test_status.unexpected == 0


class TestStatus:
    """Class that stores information on the results of test runs for later reference"""
    def __init__(self):
        self.total_tests = 0
        self.skipped = 0
        self.unexpected = 0
        self.unexpected_pass = 0
        self.repeated_runs = 0
        self.expected_repeated_runs = 0
        self.all_skipped = False


def run_tests(config, test_paths, product, **kwargs):
    """Set up the test environment, load the list of tests to be executed, and
    invoke the remainder of the code to execute tests"""
    mp = mpcontext.get_context()
    if kwargs["instrument_to_file"] is None:
        recorder = instruments.NullInstrument()
    else:
        recorder = instruments.Instrument(kwargs["instrument_to_file"])
    with recorder as recording, capture.CaptureIO(logger,
                                                  not kwargs["no_capture_stdio"],
                                                  mp_context=mp):
        recording.set(["startup"])
        env.do_delayed_imports(logger, test_paths)

        product = products.Product(config, product)

        env_extras = product.get_env_extras(**kwargs)

        product.check_args(**kwargs)

        if kwargs["install_fonts"]:
            env_extras.append(FontInstaller(
                logger,
                font_dir=kwargs["font_dir"],
                ahem=os.path.join(test_paths["/"]["tests_path"], "fonts/Ahem.ttf")
            ))

        recording.set(["startup", "load_tests"])

        test_groups = (testloader.TestGroupsFile(logger, kwargs["test_groups_file"])
                       if kwargs["test_groups_file"] else None)

        (test_source_cls,
         test_source_kwargs,
         chunker_kwargs) = testloader.get_test_src(logger=logger,
                                                   test_groups=test_groups,
                                                   **kwargs)
        run_info, test_loader = get_loader(test_paths,
                                           product.name,
                                           run_info_extras=product.run_info_extras(**kwargs),
                                           chunker_kwargs=chunker_kwargs,
                                           test_groups=test_groups,
                                           **kwargs)

        logger.info("Using %i client processes" % kwargs["processes"])

        test_status = TestStatus()
        repeat = kwargs["repeat"]
        test_status.expected_repeated_runs = repeat

        if len(test_loader.test_ids) == 0 and kwargs["test_list"]:
            logger.critical("Unable to find any tests at the path(s):")
            for path in kwargs["test_list"]:
                logger.critical("  %s" % path)
            logger.critical("Please check spelling and make sure there are tests in the specified path(s).")
            return False, test_status
        kwargs["pause_after_test"] = get_pause_after_test(test_loader, **kwargs)

        ssl_config = {"type": kwargs["ssl_type"],
                      "openssl": {"openssl_binary": kwargs["openssl_binary"]},
                      "pregenerated": {"host_key_path": kwargs["host_key_path"],
                                       "host_cert_path": kwargs["host_cert_path"],
                                       "ca_cert_path": kwargs["ca_cert_path"]}}

        testharness_timeout_multipler = product.get_timeout_multiplier("testharness",
                                                                       run_info,
                                                                       **kwargs)

        mojojs_path = kwargs["mojojs_path"] if kwargs["enable_mojojs"] else None
        inject_script = kwargs["inject_script"] if kwargs["inject_script"] else None

        recording.set(["startup", "start_environment"])
        with env.TestEnvironment(test_paths,
                                 testharness_timeout_multipler,
                                 kwargs["pause_after_test"],
                                 kwargs["debug_test"],
                                 kwargs["debug_info"],
                                 product.env_options,
                                 ssl_config,
                                 env_extras,
                                 kwargs["enable_webtransport_h3"],
                                 mojojs_path,
                                 inject_script) as test_environment:
            recording.set(["startup", "ensure_environment"])
            try:
                test_environment.ensure_started()
                start_time = datetime.now()
            except env.TestEnvironmentError as e:
                logger.critical("Error starting test environment: %s" % e)
                raise

            recording.set(["startup"])

            max_time = None
            if "repeat_max_time" in kwargs:
                max_time = timedelta(minutes=kwargs["repeat_max_time"])

            repeat_until_unexpected = kwargs["repeat_until_unexpected"]

            # keep track of longest time taken to complete a test suite iteration
            # so that the runs can be stopped to avoid a possible TC timeout.
            longest_iteration_time = timedelta()

            while test_status.repeated_runs < repeat or repeat_until_unexpected:
                # if the next repeat run could cause the TC timeout to be reached,
                # stop now and use the test results we have.
                # Pad the total time by 10% to ensure ample time for the next iteration(s).
                estimate = (datetime.now() +
                            timedelta(seconds=(longest_iteration_time.total_seconds() * 1.1)))
                if not repeat_until_unexpected and max_time and estimate >= start_time + max_time:
                    logger.info(f"Ran {test_status.repeated_runs} of {repeat} iterations.")
                    break

                # begin tracking runtime of the test suite
                iteration_start = datetime.now()
                test_status.repeated_runs += 1
                if repeat_until_unexpected:
                    logger.info(f"Repetition {test_status.repeated_runs}")
                elif repeat > 1:
                    logger.info(f"Repetition {test_status.repeated_runs} / {repeat}")

                iter_success = run_test_iteration(test_status, test_loader, test_source_kwargs,
                                                  test_source_cls, run_info, recording,
                                                  test_environment, product, kwargs)
                # if there were issues with the suite run(tests not loaded, etc.) return
                if not iter_success:
                    return False, test_status
                recording.set(["after-end"])
                logger.info(f"Got {test_status.unexpected} unexpected results, "
                    f"with {test_status.unexpected_pass} unexpected passes")
                logger.suite_end()

                # Note this iteration's runtime
                iteration_runtime = datetime.now() - iteration_start
                # determine the longest test suite runtime seen.
                longest_iteration_time = max(longest_iteration_time,
                                             iteration_runtime)

                if repeat_until_unexpected and test_status.unexpected > 0:
                    break
                if test_status.repeated_runs == 1 and len(test_loader.test_ids) == test_status.skipped:
                    test_status.all_skipped = True
                    break

    # Return the evaluation of the runs and the number of repeated iterations that were run.
    return evaluate_runs(test_status, kwargs), test_status


def check_stability(**kwargs):
    from . import stability
    if kwargs["stability"]:
        logger.warning("--stability is deprecated; please use --verify instead!")
        kwargs['verify_max_time'] = None
        kwargs['verify_chaos_mode'] = False
        kwargs['verify_repeat_loop'] = 0
        kwargs['verify_repeat_restart'] = 10 if kwargs['repeat'] == 1 else kwargs['repeat']
        kwargs['verify_output_results'] = True

    return stability.check_stability(logger,
                                     max_time=kwargs['verify_max_time'],
                                     chaos_mode=kwargs['verify_chaos_mode'],
                                     repeat_loop=kwargs['verify_repeat_loop'],
                                     repeat_restart=kwargs['verify_repeat_restart'],
                                     output_results=kwargs['verify_output_results'],
                                     **kwargs)


def start(**kwargs):
    assert logger is not None

    logged_critical = wptlogging.LoggedAboveLevelHandler("CRITICAL")
    handler = handlers.LogLevelFilter(logged_critical, "CRITICAL")
    logger.add_handler(handler)

    rv = False
    try:
        if kwargs["list_test_groups"]:
            list_test_groups(**kwargs)
        elif kwargs["list_disabled"]:
            list_disabled(**kwargs)
        elif kwargs["list_tests"]:
            list_tests(**kwargs)
        elif kwargs["verify"] or kwargs["stability"]:
            rv = check_stability(**kwargs) or logged_critical.has_log
        else:
            rv = not run_tests(**kwargs)[0] or logged_critical.has_log
    finally:
        logger.shutdown()
        logger.remove_handler(handler)
    return rv


def main():
    """Main entry point when calling from the command line"""
    kwargs = wptcommandline.parse_args()

    try:
        if kwargs["prefs_root"] is None:
            kwargs["prefs_root"] = os.path.abspath(os.path.join(here, "prefs"))

        setup_logging(kwargs, {"raw": sys.stdout})

        return start(**kwargs)
    except Exception:
        if kwargs["pdb"]:
            import pdb
            import traceback
            print(traceback.format_exc())
            pdb.post_mortem()
        else:
            raise
