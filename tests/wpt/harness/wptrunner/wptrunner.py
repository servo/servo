# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

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
from testrunner import ManagerGroup

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

def get_loader(test_paths, product, ssl_env, debug=False, **kwargs):
    run_info = wpttest.get_run_info(kwargs["run_info"], product, debug=debug)

    test_manifests = testloader.ManifestLoader(test_paths, force_manifest_update=kwargs["manifest_update"]).load()

    test_filter = testloader.TestFilter(include=kwargs["include"],
                                        exclude=kwargs["exclude"],
                                        manifest_path=kwargs["include_manifest"],
                                        test_manifests=test_manifests)

    test_loader = testloader.TestLoader(test_manifests,
                                        kwargs["test_types"],
                                        test_filter,
                                        run_info,
                                        chunk_type=kwargs["chunk_type"],
                                        total_chunks=kwargs["total_chunks"],
                                        chunk_number=kwargs["this_chunk"],
                                        include_https=ssl_env.ssl_enabled)
    return run_info, test_loader

def list_test_groups(test_paths, product, **kwargs):
    env.do_delayed_imports(logger, test_paths)

    ssl_env = env.ssl_env(logger, **kwargs)

    run_info, test_loader = get_loader(test_paths, product, ssl_env,
                                       **kwargs)

    for item in sorted(test_loader.groups(kwargs["test_types"])):
        print item


def list_disabled(test_paths, product, **kwargs):
    env.do_delayed_imports(logger, test_paths)

    rv = []

    ssl_env = env.ssl_env(logger, **kwargs)

    run_info, test_loader = get_loader(test_paths, product, ssl_env,
                                       **kwargs)

    for test_type, tests in test_loader.disabled_tests.iteritems():
        for test in tests:
            rv.append({"test": test.id, "reason": test.disabled()})
    print json.dumps(rv, indent=2)


def get_pause_after_test(test_loader, **kwargs):
    total_tests = sum(len(item) for item in test_loader.tests.itervalues())
    if kwargs["pause_after_test"] is None:
        if kwargs["repeat"] == 1 and total_tests == 1:
            return True
        return False
    return kwargs["pause_after_test"]


def run_tests(config, test_paths, product, **kwargs):
    with wptlogging.CaptureIO(logger, not kwargs["no_capture_stdio"]):
        env.do_delayed_imports(logger, test_paths)

        (check_args,
         browser_cls, get_browser_kwargs,
         executor_classes, get_executor_kwargs,
         env_options) = products.load_product(config, product)

        ssl_env = env.ssl_env(logger, **kwargs)

        check_args(**kwargs)

        if "test_loader" in kwargs:
            run_info = wpttest.get_run_info(kwargs["run_info"], product, debug=False)
            test_loader = kwargs["test_loader"]
        else:
            run_info, test_loader = get_loader(test_paths, product, ssl_env,
                                               **kwargs)

        if kwargs["run_by_dir"] is False:
            test_source_cls = testloader.SingleTestSource
            test_source_kwargs = {}
        else:
            # A value of None indicates infinite depth
            test_source_cls = testloader.PathGroupedSource
            test_source_kwargs = {"depth": kwargs["run_by_dir"]}

        logger.info("Using %i client processes" % kwargs["processes"])

        unexpected_total = 0

        kwargs["pause_after_test"] = get_pause_after_test(test_loader, **kwargs)

        with env.TestEnvironment(test_paths,
                                 ssl_env,
                                 kwargs["pause_after_test"],
                                 kwargs["debug_info"],
                                 env_options) as test_environment:
            try:
                test_environment.ensure_started()
            except env.TestEnvironmentError as e:
                logger.critical("Error starting test environment: %s" % e.message)
                raise

            browser_kwargs = get_browser_kwargs(ssl_env=ssl_env, **kwargs)

            repeat = kwargs["repeat"]
            for repeat_count in xrange(repeat):
                if repeat > 1:
                    logger.info("Repetition %i / %i" % (repeat_count + 1, repeat))


                unexpected_count = 0
                logger.suite_start(test_loader.test_ids, run_info)
                for test_type in kwargs["test_types"]:
                    logger.info("Running %s tests" % test_type)

                    for test in test_loader.disabled_tests[test_type]:
                        logger.test_start(test.id)
                        logger.test_end(test.id, status="SKIP")

                    executor_cls = executor_classes.get(test_type)
                    executor_kwargs = get_executor_kwargs(test_type,
                                                          test_environment.external_config,
                                                          test_environment.cache_manager,
                                                          **kwargs)

                    if executor_cls is None:
                        logger.error("Unsupported test type %s for product %s" %
                                     (test_type, product))
                        continue


                    with ManagerGroup("web-platform-tests",
                                      kwargs["processes"],
                                      test_source_cls,
                                      test_source_kwargs,
                                      browser_cls,
                                      browser_kwargs,
                                      executor_cls,
                                      executor_kwargs,
                                      kwargs["pause_after_test"],
                                      kwargs["pause_on_unexpected"],
                                      kwargs["debug_info"]) as manager_group:
                        try:
                            manager_group.run(test_type, test_loader.tests)
                        except KeyboardInterrupt:
                            logger.critical("Main thread got signal")
                            manager_group.stop()
                            raise
                    unexpected_count += manager_group.unexpected_count()

                unexpected_total += unexpected_count
                logger.info("Got %i unexpected results" % unexpected_count)
                logger.suite_end()

    return unexpected_total == 0


def main():
    """Main entry point when calling from the command line"""
    try:
        kwargs = wptcommandline.parse_args()

        if kwargs["prefs_root"] is None:
            kwargs["prefs_root"] = os.path.abspath(os.path.join(here, "prefs"))

        setup_logging(kwargs, {"raw": sys.stdout})

        if kwargs["list_test_groups"]:
            list_test_groups(**kwargs)
        elif kwargs["list_disabled"]:
            list_disabled(**kwargs)
        else:
            return run_tests(**kwargs)
    except Exception:
        import pdb, traceback
        print traceback.format_exc()
        pdb.post_mortem()
