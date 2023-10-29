# mypy: allow-untyped-calls, allow-untyped-defs

import json
import os
import signal
import sys
from collections import defaultdict
from datetime import datetime, timedelta
from typing import Any, Tuple

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
from .testrunner import ManagerGroup, TestImplementation

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


def get_loader(test_paths: wptcommandline.TestPaths,
               product: products.Product,
               **kwargs: Any) -> Tuple[testloader.TestQueueBuilder, testloader.TestLoader]:
    run_info_extras = product.run_info_extras(**kwargs)
    base_run_info = wpttest.get_run_info(kwargs["run_info"],
                                         product.name,
                                         browser_version=kwargs.get("browser_version"),
                                         browser_channel=kwargs.get("browser_channel"),
                                         verify=kwargs.get("verify"),
                                         debug=kwargs["debug"],
                                         extras=run_info_extras,
                                         device_serials=kwargs.get("device_serial"),
                                         adb_binary=kwargs.get("adb_binary"))

    subsuites = testloader.load_subsuites(logger,
                                          base_run_info,
                                          kwargs["subsuite_file"],
                                          set(kwargs["subsuites"] or []))

    if kwargs["test_groups_file"] is not None:
        test_groups = testloader.TestGroups(logger,
                                            kwargs["test_groups_file"],
                                            subsuites)
    else:
        test_groups = None

    test_manifests = testloader.ManifestLoader(test_paths,
                                               force_manifest_update=kwargs["manifest_update"],
                                               manifest_download=kwargs["manifest_download"]).load()

    manifest_filters = []
    test_filters = []

    include = kwargs["include"]
    if kwargs["include_file"]:
        include = include or []
        include.extend(testloader.read_include_from_file(kwargs["include_file"]))
    if test_groups:
        include = testloader.update_include_for_groups(test_groups, include)

    if kwargs["tags"] or kwargs["exclude_tags"]:
        test_filters.append(testloader.TagFilter(kwargs["tags"], kwargs["exclude_tags"]))

    if include or kwargs["exclude"] or kwargs["include_manifest"] or kwargs["default_exclude"]:
        manifest_filters.append(testloader.TestFilter(include=include,
                                                      exclude=kwargs["exclude"],
                                                      manifest_path=kwargs["include_manifest"],
                                                      test_manifests=test_manifests,
                                                      explicit=kwargs["default_exclude"]))

    ssl_enabled = sslutils.get_cls(kwargs["ssl_type"]).ssl_enabled
    h2_enabled = wptserve.utils.http2_compatible()

    test_queue_builder, chunker_kwargs = testloader.get_test_queue_builder(logger=logger,
                                                                           test_groups=test_groups,
                                                                           **kwargs)

    test_loader = testloader.TestLoader(test_manifests=test_manifests,
                                        test_types=kwargs["test_types"],
                                        base_run_info=base_run_info,
                                        subsuites=subsuites,
                                        manifest_filters=manifest_filters,
                                        test_filters=test_filters,
                                        chunk_type=kwargs["chunk_type"],
                                        total_chunks=kwargs["total_chunks"],
                                        chunk_number=kwargs["this_chunk"],
                                        include_https=ssl_enabled,
                                        include_h2=h2_enabled,
                                        include_webtransport_h3=kwargs["enable_webtransport_h3"],
                                        skip_timeout=kwargs["skip_timeout"],
                                        skip_crash=kwargs["skip_crash"],
                                        skip_implementation_status=kwargs["skip_implementation_status"],
                                        chunker_kwargs=chunker_kwargs)
    return test_queue_builder, test_loader


def list_test_groups(test_paths, product, **kwargs):
    env.do_delayed_imports(logger, test_paths)

    _, test_loader = get_loader(test_paths,
                                product,
                                **kwargs)

    for item in sorted(test_loader.groups(kwargs["test_types"])):
        print(item)


def list_disabled(test_paths, product, **kwargs):
    env.do_delayed_imports(logger, test_paths)

    rv = []

    _, test_loader = get_loader(test_paths, product, **kwargs)

    for test_type, tests in test_loader.disabled_tests.items():
        for test in tests:
            rv.append({"test": test.id, "reason": test.disabled()})
    print(json.dumps(rv, indent=2))


def list_tests(test_paths, product, **kwargs):
    env.do_delayed_imports(logger, test_paths)

    _, test_loader = get_loader(test_paths, product, **kwargs)

    for test in test_loader.test_ids:
        print(test)


def get_pause_after_test(test_loader, **kwargs):
    if kwargs["pause_after_test"] is not None:
        return kwargs["pause_after_test"]
    if kwargs["repeat_until_unexpected"]:
        return False
    if kwargs["headless"]:
        return False
    if kwargs["debug_test"]:
        return True
    tests = test_loader.tests
    is_single_testharness = True
    testharness_count = 0
    for tests_by_type in tests.values():
        for test_type, tests in tests_by_type.items():
            if test_type != "testharness" and len(tests):
                is_single_testharness = False
                break
            elif test_type == "testharness":
                testharness_count += len(tests)
                if testharness_count > 1:
                    is_single_testharness = False
                    break
    return kwargs["repeat"] == 1 and kwargs["rerun"] == 1 and is_single_testharness




def log_suite_start(tests_by_group, base_run_info, subsuites, run_by_dir):
    logger.suite_start(tests_by_group,
                       name='web-platform-test',
                       run_info=base_run_info,
                       extra={"run_by_dir": run_by_dir})

    for name, subsuite in subsuites.items():
        logger.add_subsuite(name=name, run_info=subsuite.run_info_extras)


def run_test_iteration(test_status, test_loader, test_queue_builder,
                       recording, test_environment, product, kwargs):
    """Runs the entire test suite.
    This is called for each repeat run requested."""
    tests_by_type = defaultdict(list)

    for test_type in test_loader.test_types:
        for subsuite_name, subsuite in test_loader.subsuites.items():
            type_tests_active = test_loader.tests[subsuite_name][test_type]
            type_tests_disabled = test_loader.disabled_tests[subsuite_name][test_type]
            if type_tests_active or type_tests_disabled:
                tests_by_type[(subsuite_name, test_type)].extend(type_tests_active)
                tests_by_type[(subsuite_name, test_type)].extend(type_tests_disabled)

    tests_by_group = test_queue_builder.tests_by_group(tests_by_type)

    log_suite_start(tests_by_group,
                    test_loader.base_run_info,
                    test_loader.subsuites,
                    kwargs["run_by_dir"])

    test_implementations = {}
    tests_to_run = defaultdict(list)

    for test_type in test_loader.test_types:
        executor_cls = product.executor_classes.get(test_type)
        if executor_cls is None:
            logger.warning(f"Unsupported test type {test_type} for product {product.name}")
            continue
        browser_cls = product.get_browser_cls(test_type)

        for subsuite_name, subsuite in test_loader.subsuites.items():
            if (subsuite_name, test_type) not in tests_by_type:
                continue
            run_info = subsuite.run_info
            executor_kwargs = product.get_executor_kwargs(logger,
                                                          test_type,
                                                          test_environment,
                                                          run_info,
                                                          subsuite=subsuite,
                                                          **kwargs)
            browser_kwargs = product.get_browser_kwargs(logger,
                                                        test_type,
                                                        run_info,
                                                        config=test_environment.config,
                                                        num_test_groups=len(tests_by_group),
                                                        subsuite=subsuite,
                                                        **kwargs)

            test_implementations[(subsuite_name, test_type)] = TestImplementation(executor_cls,
                                                                                  executor_kwargs,
                                                                                  browser_cls,
                                                                                  browser_kwargs)

            for test in test_loader.disabled_tests[subsuite_name][test_type]:
                logger.test_start(test.id, subsuite=subsuite_name)
                logger.test_end(test.id, status="SKIP", subsuite=subsuite_name)
                test_status.skipped += 1

            if test_type == "testharness":
                for test in test_loader.tests[subsuite_name][test_type]:
                    skip_reason = None
                    if test.testdriver and not executor_cls.supports_testdriver:
                        skip_reason = "Executor does not support testdriver.js"
                    elif test.jsshell and not executor_cls.supports_jsshell:
                        skip_reason = "Executor does not support jsshell"
                    if skip_reason:
                        logger.test_start(test.id, subsuite=subsuite_name)
                        logger.test_end(test.id,
                                        status="SKIP",
                                        subsuite=subsuite_name,
                                        message=skip_reason)
                        test_status.skipped += 1
                    else:
                        tests_to_run[(subsuite_name, test_type)].append(test)
            else:
                tests_to_run[(subsuite_name, test_type)] = test_loader.tests[subsuite_name][test_type]

    unexpected_fail_tests = defaultdict(list)
    unexpected_pass_tests = defaultdict(list)
    recording.pause()
    retry_counts = kwargs["retry_unexpected"]
    for retry_index in range(retry_counts + 1):
        if retry_index > 0:
            if kwargs["fail_on_unexpected_pass"]:
                for (subtests, test_type), tests in unexpected_pass_tests.items():
                    unexpected_fail_tests[(subtests, test_type)].extend(tests)
            tests_to_run = unexpected_fail_tests
            if sum(len(tests) for tests in tests_to_run.values()) == 0:
                break
            tests_by_group = test_queue_builder.tests_by_group(tests_to_run)

            logger.suite_end()

            log_suite_start(tests_by_group,
                            test_loader.base_run_info,
                            test_loader.subsuites,
                            kwargs["run_by_dir"])

        with ManagerGroup("web-platform-tests",
                          test_queue_builder,
                          test_implementations,
                          retry_index,
                          kwargs["rerun"],
                          kwargs["pause_after_test"],
                          kwargs["pause_on_unexpected"],
                          kwargs["restart_on_unexpected"],
                          kwargs["debug_info"],
                          not kwargs["no_capture_stdio"],
                          kwargs["restart_on_new_group"],
                          recording=recording,
                          max_restarts=kwargs["max_restarts"],
                          ) as manager_group:
            try:
                handle_interrupt_signals()
                manager_group.run(tests_to_run)
            except KeyboardInterrupt:
                logger.critical(
                    "Main thread got signal; "
                    "waiting for TestRunnerManager threads to exit.")
                manager_group.stop()
                manager_group.wait(timeout=10)
                raise

            test_status.total_tests += manager_group.test_count()
            unexpected_fail_tests = manager_group.unexpected_fail_tests()
            unexpected_pass_tests = manager_group.unexpected_pass_tests()

    test_status.unexpected_pass += sum(len(tests) for tests in unexpected_pass_tests.values())
    test_status.unexpected += sum(len(tests) for tests in unexpected_pass_tests.values())
    test_status.unexpected += sum(len(tests) for tests in unexpected_fail_tests.values())
    logger.suite_end()
    return True


def handle_interrupt_signals():
    def termination_handler(_signum, _unused_frame):
        raise KeyboardInterrupt()
    if sys.platform == "win32":
        signal.signal(signal.SIGBREAK, termination_handler)
    else:
        signal.signal(signal.SIGTERM, termination_handler)


def evaluate_runs(test_status, **kwargs):
    """Evaluates the test counts after the given number of repeat runs has finished"""
    if test_status.total_tests == 0:
        if test_status.skipped > 0:
            logger.warning("All requested tests were skipped")
        else:
            if kwargs["default_exclude"]:
                logger.info("No tests ran")
                return True
            else:
                logger.critical("No tests ran")
                return False

    if test_status.unexpected and not kwargs["fail_on_unexpected"]:
        logger.info(f"Tolerating {test_status.unexpected} unexpected results")
        return True

    all_unexpected_passed = (test_status.unexpected and
                             test_status.unexpected == test_status.unexpected_pass)
    if all_unexpected_passed and not kwargs["fail_on_unexpected_pass"]:
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


def run_tests(config, product, test_paths, **kwargs):
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

        env_extras = product.get_env_extras(**kwargs)

        product.check_args(**kwargs)

        if kwargs["install_fonts"]:
            env_extras.append(FontInstaller(
                logger,
                font_dir=kwargs["font_dir"],
                ahem=os.path.join(test_paths["/"].tests_path, "fonts/Ahem.ttf")
            ))

        recording.set(["startup", "load_tests"])

        test_queue_builder, test_loader = get_loader(test_paths,
                                                     product,
                                                     **kwargs)

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

        # testharness.js is global so we can't set the timeout multiplier in that file by subsuite
        testharness_timeout_multipler = product.get_timeout_multiplier("testharness",
                                                                       test_loader.base_run_info,
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
                                 inject_script,
                                 kwargs["suppress_handler_traceback"]) as test_environment:
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

                iter_success = run_test_iteration(test_status,
                                                  test_loader,
                                                  test_queue_builder,
                                                  recording,
                                                  test_environment,
                                                  product,
                                                  kwargs)
                # if there were issues with the suite run(tests not loaded, etc.) return
                if not iter_success:
                    return False, test_status
                recording.set(["after-end"])
                logger.info(f"Got {test_status.unexpected} unexpected results, "
                    f"with {test_status.unexpected_pass} unexpected passes")

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
    return evaluate_runs(test_status, **kwargs), test_status


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
