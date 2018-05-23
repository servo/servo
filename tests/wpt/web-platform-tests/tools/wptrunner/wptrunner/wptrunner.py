from __future__ import unicode_literals

import json
import os
import sys

import environment as env
import products
import testloader
import wptcommandline
import wptlogging
import wpttest
from font import FontInstaller
from testrunner import ManagerGroup
from browsers.base import NullBrowser

here = os.path.split(__file__)[0]

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


def get_loader(test_paths, product, ssl_env, debug=None, run_info_extras=None, **kwargs):
    if run_info_extras is None:
        run_info_extras = {}

    run_info = wpttest.get_run_info(kwargs["run_info"], product,
                                    browser_version=kwargs.get("browser_version"),
                                    debug=debug,
                                    extras=run_info_extras)

    test_manifests = testloader.ManifestLoader(test_paths, force_manifest_update=kwargs["manifest_update"],
                                               manifest_download=kwargs["manifest_download"]).load()

    manifest_filters = []
    meta_filters = []

    if kwargs["include"] or kwargs["exclude"] or kwargs["include_manifest"]:
        manifest_filters.append(testloader.TestFilter(include=kwargs["include"],
                                                      exclude=kwargs["exclude"],
                                                      manifest_path=kwargs["include_manifest"],
                                                      test_manifests=test_manifests))
    if kwargs["tags"]:
        meta_filters.append(testloader.TagFilter(tags=kwargs["tags"]))

    test_loader = testloader.TestLoader(test_manifests,
                                        kwargs["test_types"],
                                        run_info,
                                        manifest_filters=manifest_filters,
                                        meta_filters=meta_filters,
                                        chunk_type=kwargs["chunk_type"],
                                        total_chunks=kwargs["total_chunks"],
                                        chunk_number=kwargs["this_chunk"],
                                        include_https=ssl_env.ssl_enabled,
                                        skip_timeout=kwargs["skip_timeout"])
    return run_info, test_loader


def list_test_groups(test_paths, product, **kwargs):
    env.do_delayed_imports(logger, test_paths)

    ssl_env = env.ssl_env(logger, **kwargs)

    run_info_extras = products.load_product(kwargs["config"], product)[-1](**kwargs)

    run_info, test_loader = get_loader(test_paths, product, ssl_env,
                                       run_info_extras=run_info_extras, **kwargs)

    for item in sorted(test_loader.groups(kwargs["test_types"])):
        print item


def list_disabled(test_paths, product, **kwargs):
    env.do_delayed_imports(logger, test_paths)

    rv = []

    run_info_extras = products.load_product(kwargs["config"], product)[-1](**kwargs)

    ssl_env = env.ssl_env(logger, **kwargs)

    run_info, test_loader = get_loader(test_paths, product, ssl_env,
                                       run_info_extras=run_info_extras, **kwargs)

    for test_type, tests in test_loader.disabled_tests.iteritems():
        for test in tests:
            rv.append({"test": test.id, "reason": test.disabled()})
    print json.dumps(rv, indent=2)


def list_tests(test_paths, product, **kwargs):
    env.do_delayed_imports(logger, test_paths)

    ssl_env = env.ssl_env(logger, **kwargs)

    run_info_extras = products.load_product(kwargs["config"], product)[-1](**kwargs)

    run_info, test_loader = get_loader(test_paths, product, ssl_env,
                                       run_info_extras=run_info_extras, **kwargs)

    for test in test_loader.test_ids:
        print test


def get_pause_after_test(test_loader, **kwargs):
    total_tests = sum(len(item) for item in test_loader.tests.itervalues())
    if kwargs["pause_after_test"] is None:
        if kwargs["repeat_until_unexpected"]:
            return False
        if kwargs["repeat"] == 1 and kwargs["rerun"] == 1 and total_tests == 1:
            return True
        return False
    return kwargs["pause_after_test"]


def run_tests(config, test_paths, product, **kwargs):
    with wptlogging.CaptureIO(logger, not kwargs["no_capture_stdio"]):
        env.do_delayed_imports(logger, test_paths)

        (check_args,
         target_browser_cls, get_browser_kwargs,
         executor_classes, get_executor_kwargs,
         env_options, get_env_extras, run_info_extras) = products.load_product(config, product)

        ssl_env = env.ssl_env(logger, **kwargs)
        env_extras = get_env_extras(**kwargs)

        check_args(**kwargs)

        if kwargs["install_fonts"]:
            env_extras.append(FontInstaller(
                font_dir=kwargs["font_dir"],
                ahem=os.path.join(kwargs["tests_root"], "fonts/Ahem.ttf")
            ))

        if "test_loader" in kwargs:
            run_info = wpttest.get_run_info(kwargs["run_info"], product,
                                            browser_version=kwargs.get("browser_version"),
                                            debug=None,
                                            extras=run_info_extras(**kwargs))
            test_loader = kwargs["test_loader"]
        else:
            run_info, test_loader = get_loader(test_paths,
                                               product,
                                               ssl_env,
                                               run_info_extras=run_info_extras(**kwargs),
                                               **kwargs)

        test_source_kwargs = {"processes": kwargs["processes"]}
        if kwargs["run_by_dir"] is False:
            test_source_cls = testloader.SingleTestSource
        else:
            # A value of None indicates infinite depth
            test_source_cls = testloader.PathGroupedSource
            test_source_kwargs["depth"] = kwargs["run_by_dir"]

        logger.info("Using %i client processes" % kwargs["processes"])

        test_total = 0
        unexpected_total = 0

        kwargs["pause_after_test"] = get_pause_after_test(test_loader, **kwargs)

        with env.TestEnvironment(test_paths,
                                 ssl_env,
                                 kwargs["pause_after_test"],
                                 kwargs["debug_info"],
                                 env_options,
                                 env_extras) as test_environment:
            try:
                test_environment.ensure_started()
            except env.TestEnvironmentError as e:
                logger.critical("Error starting test environment: %s" % e.message)
                raise

            repeat = kwargs["repeat"]
            repeat_count = 0
            repeat_until_unexpected = kwargs["repeat_until_unexpected"]

            while repeat_count < repeat or repeat_until_unexpected:
                repeat_count += 1
                if repeat_until_unexpected:
                    logger.info("Repetition %i" % (repeat_count))
                elif repeat > 1:
                    logger.info("Repetition %i / %i" % (repeat_count, repeat))

                test_count = 0
                unexpected_count = 0
                logger.suite_start(test_loader.test_ids, name='web-platform-test', run_info=run_info)
                for test_type in kwargs["test_types"]:
                    logger.info("Running %s tests" % test_type)

                    # WebDriver tests may create and destroy multiple browser
                    # processes as part of their expected behavior. These
                    # processes are managed by a WebDriver server binary. This
                    # obviates the need for wptrunner to provide a browser, so
                    # the NullBrowser is used in place of the "target" browser
                    if test_type == "wdspec":
                        browser_cls = NullBrowser
                    else:
                        browser_cls = target_browser_cls

                    browser_kwargs = get_browser_kwargs(test_type,
                                                        run_info,
                                                        ssl_env=ssl_env,
                                                        config=test_environment.config,
                                                        **kwargs)

                    executor_cls = executor_classes.get(test_type)
                    executor_kwargs = get_executor_kwargs(test_type,
                                                          test_environment.config,
                                                          test_environment.cache_manager,
                                                          run_info,
                                                          **kwargs)

                    if executor_cls is None:
                        logger.error("Unsupported test type %s for product %s" %
                                     (test_type, product))
                        continue

                    for test in test_loader.disabled_tests[test_type]:
                        logger.test_start(test.id)
                        logger.test_end(test.id, status="SKIP")

                    if test_type == "testharness":
                        run_tests = {"testharness": []}
                        for test in test_loader.tests["testharness"]:
                            if test.testdriver and not executor_cls.supports_testdriver:
                                logger.test_start(test.id)
                                logger.test_end(test.id, status="SKIP")
                            elif test.jsshell and not executor_cls.supports_jsshell:
                                # We expect that tests for JavaScript shells
                                # will not be run along with tests that run in
                                # a full web browser, so we silently skip them
                                # here.
                                pass
                            else:
                                run_tests["testharness"].append(test)
                    else:
                        run_tests = test_loader.tests

                    with ManagerGroup("web-platform-tests",
                                      kwargs["processes"],
                                      test_source_cls,
                                      test_source_kwargs,
                                      browser_cls,
                                      browser_kwargs,
                                      executor_cls,
                                      executor_kwargs,
                                      kwargs["rerun"],
                                      kwargs["pause_after_test"],
                                      kwargs["pause_on_unexpected"],
                                      kwargs["restart_on_unexpected"],
                                      kwargs["debug_info"]) as manager_group:
                        try:
                            manager_group.run(test_type, run_tests)
                        except KeyboardInterrupt:
                            logger.critical("Main thread got signal")
                            manager_group.stop()
                            raise
                    test_count += manager_group.test_count()
                    unexpected_count += manager_group.unexpected_count()

                test_total += test_count
                unexpected_total += unexpected_count
                logger.info("Got %i unexpected results" % unexpected_count)
                if repeat_until_unexpected and unexpected_total > 0:
                    break
                logger.suite_end()

    if test_total == 0:
        logger.error("No tests ran")
        return False

    if unexpected_total and not kwargs["fail_on_unexpected"]:
        logger.info("Tolerating %s unexpected results" % unexpected_total)
        return True

    return unexpected_total == 0


def check_stability(**kwargs):
    import stability
    return stability.check_stability(logger,
                                     max_time=kwargs['verify_max_time'],
                                     chaos_mode=kwargs['verify_chaos_mode'],
                                     repeat_loop=kwargs['verify_repeat_loop'],
                                     repeat_restart=kwargs['verify_repeat_restart'],
                                     output_results=kwargs['verify_output_results'],
                                     **kwargs)

def start(**kwargs):
    if kwargs["list_test_groups"]:
        list_test_groups(**kwargs)
    elif kwargs["list_disabled"]:
        list_disabled(**kwargs)
    elif kwargs["list_tests"]:
        list_tests(**kwargs)
    elif kwargs["verify"]:
        check_stability(**kwargs)
    else:
        return not run_tests(**kwargs)


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
            print traceback.format_exc()
            pdb.post_mortem()
        else:
            raise
