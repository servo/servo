from __future__ import print_function, unicode_literals

import json
import os
import sys

from wptserve import sslutils

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
    return logger


def get_loader(test_paths, product, debug=None, run_info_extras=None, **kwargs):
    if run_info_extras is None:
        run_info_extras = {}

    run_info = wpttest.get_run_info(kwargs["run_info"], product,
                                    browser_version=kwargs.get("browser_version"),
                                    browser_channel=kwargs.get("browser_channel"),
                                    verify=kwargs.get("verify"),
                                    debug=debug,
                                    extras=run_info_extras)

    test_manifests = testloader.ManifestLoader(test_paths, force_manifest_update=kwargs["manifest_update"],
                                               manifest_download=kwargs["manifest_download"]).load()

    manifest_filters = []

    if kwargs["include"] or kwargs["exclude"] or kwargs["include_manifest"] or kwargs["default_exclude"]:
        manifest_filters.append(testloader.TestFilter(include=kwargs["include"],
                                                      exclude=kwargs["exclude"],
                                                      manifest_path=kwargs["include_manifest"],
                                                      test_manifests=test_manifests,
                                                      explicit=kwargs["default_exclude"]))

    ssl_enabled = sslutils.get_cls(kwargs["ssl_type"]).ssl_enabled
    test_loader = testloader.TestLoader(test_manifests,
                                        kwargs["test_types"],
                                        run_info,
                                        manifest_filters=manifest_filters,
                                        chunk_type=kwargs["chunk_type"],
                                        total_chunks=kwargs["total_chunks"],
                                        chunk_number=kwargs["this_chunk"],
                                        include_https=ssl_enabled,
                                        skip_timeout=kwargs["skip_timeout"])
    return run_info, test_loader


def list_test_groups(test_paths, product, **kwargs):
    env.do_delayed_imports(logger, test_paths)

    run_info_extras = products.load_product(kwargs["config"], product)[-1](**kwargs)

    run_info, test_loader = get_loader(test_paths, product,
                                       run_info_extras=run_info_extras, **kwargs)

    for item in sorted(test_loader.groups(kwargs["test_types"])):
        print(item)


def list_disabled(test_paths, product, **kwargs):
    env.do_delayed_imports(logger, test_paths)

    rv = []

    run_info_extras = products.load_product(kwargs["config"], product)[-1](**kwargs)

    run_info, test_loader = get_loader(test_paths, product,
                                       run_info_extras=run_info_extras, **kwargs)

    for test_type, tests in test_loader.disabled_tests.iteritems():
        for test in tests:
            rv.append({"test": test.id, "reason": test.disabled()})
    print(json.dumps(rv, indent=2))


def list_tests(test_paths, product, **kwargs):
    env.do_delayed_imports(logger, test_paths)

    run_info_extras = products.load_product(kwargs["config"], product)[-1](**kwargs)

    run_info, test_loader = get_loader(test_paths, product,
                                       run_info_extras=run_info_extras, **kwargs)

    for test in test_loader.test_ids:
        print(test)


def get_pause_after_test(test_loader, **kwargs):
    total_tests = sum(len(item) for item in test_loader.tests.itervalues())
    if kwargs["pause_after_test"] is None:
        if kwargs["repeat_until_unexpected"]:
            return False
        if kwargs["headless"]:
            return False
        if kwargs["repeat"] == 1 and kwargs["rerun"] == 1 and total_tests == 1:
            return True
        return False
    return kwargs["pause_after_test"]


def run_tests(config, test_paths, product, **kwargs):
    with wptlogging.CaptureIO(logger, not kwargs["no_capture_stdio"]):
        env.do_delayed_imports(logger, test_paths)

        product = products.load_product(config, product, load_cls=True)

        env_extras = product.get_env_extras(**kwargs)

        product.check_args(**kwargs)

        if kwargs["install_fonts"]:
            env_extras.append(FontInstaller(
                font_dir=kwargs["font_dir"],
                ahem=os.path.join(test_paths["/"]["tests_path"], "fonts/Ahem.ttf")
            ))

        run_info, test_loader = get_loader(test_paths,
                                           product.name,
                                           run_info_extras=product.run_info_extras(**kwargs),
                                           **kwargs)

        test_source_kwargs = {"processes": kwargs["processes"]}
        if kwargs["run_by_dir"] is False:
            test_source_cls = testloader.SingleTestSource
        else:
            # A value of None indicates infinite depth
            test_source_cls = testloader.PathGroupedSource
            test_source_kwargs["depth"] = kwargs["run_by_dir"]

        logger.info("Using %i client processes" % kwargs["processes"])

        skipped_tests = 0
        test_total = 0
        unexpected_total = 0

        if len(test_loader.test_ids) == 0 and kwargs["test_list"]:
            logger.error("Unable to find any tests at the path(s):")
            for path in kwargs["test_list"]:
                logger.error("  %s" % path)
            logger.error("Please check spelling and make sure there are tests in the specified path(s).")
            return False
        kwargs["pause_after_test"] = get_pause_after_test(test_loader, **kwargs)

        ssl_config = {"type": kwargs["ssl_type"],
                      "openssl": {"openssl_binary": kwargs["openssl_binary"]},
                      "pregenerated": {"host_key_path": kwargs["host_key_path"],
                                       "host_cert_path": kwargs["host_cert_path"],
                                       "ca_cert_path": kwargs["ca_cert_path"]}}

        testharness_timeout_multipler = product.get_timeout_multiplier("testharness", run_info, **kwargs)

        with env.TestEnvironment(test_paths,
                                 testharness_timeout_multipler,
                                 kwargs["pause_after_test"],
                                 kwargs["debug_info"],
                                 product.env_options,
                                 ssl_config,
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
                logger.suite_start(test_loader.test_ids,
                                   name='web-platform-test',
                                   run_info=run_info,
                                   extra={"run_by_dir": kwargs["run_by_dir"]})
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
                        browser_cls = product.browser_cls

                    browser_kwargs = product.get_browser_kwargs(test_type,
                                                                run_info,
                                                                config=test_environment.config,
                                                                **kwargs)

                    executor_cls = product.executor_classes.get(test_type)
                    executor_kwargs = product.get_executor_kwargs(test_type,
                                                                  test_environment.config,
                                                                  test_environment.cache_manager,
                                                                  run_info,
                                                                  **kwargs)

                    if executor_cls is None:
                        logger.error("Unsupported test type %s for product %s" %
                                     (test_type, product.name))
                        continue

                    for test in test_loader.disabled_tests[test_type]:
                        logger.test_start(test.id)
                        logger.test_end(test.id, status="SKIP")
                        skipped_tests += 1

                    if test_type == "testharness":
                        run_tests = {"testharness": []}
                        for test in test_loader.tests["testharness"]:
                            if ((test.testdriver and not executor_cls.supports_testdriver) or
                                (test.jsshell and not executor_cls.supports_jsshell)):
                                logger.test_start(test.id)
                                logger.test_end(test.id, status="SKIP")
                                skipped_tests += 1
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
                                      kwargs["debug_info"],
                                      not kwargs["no_capture_stdio"]) as manager_group:
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
                logger.suite_end()
                if repeat_until_unexpected and unexpected_total > 0:
                    break
                if repeat_count == 1 and len(test_loader.test_ids) == skipped_tests:
                    break

    if test_total == 0:
        if skipped_tests > 0:
            logger.warning("All requested tests were skipped")
        else:
            if kwargs["default_exclude"]:
                logger.info("No tests ran")
                return True
            else:
                logger.error("No tests ran")
                return False

    if unexpected_total and not kwargs["fail_on_unexpected"]:
        logger.info("Tolerating %s unexpected results" % unexpected_total)
        return True

    return unexpected_total == 0


def check_stability(**kwargs):
    import stability
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
    if kwargs["list_test_groups"]:
        list_test_groups(**kwargs)
    elif kwargs["list_disabled"]:
        list_disabled(**kwargs)
    elif kwargs["list_tests"]:
        list_tests(**kwargs)
    elif kwargs["verify"] or kwargs["stability"]:
        return check_stability(**kwargs)
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
            print(traceback.format_exc())
            pdb.post_mortem()
        else:
            raise
