import sys
import tempfile
from cStringIO import StringIO
from collections import defaultdict

import wptrunner
import wpttest

from mozlog import commandline, reader

logger = None


def setup_logging(args, defaults):
    global logger
    logger = commandline.setup_logging("web-platform-tests-unstable", args, defaults)
    wptrunner.setup_stdlib_logger()

    for name in args.keys():
        if name.startswith("log_"):
            args.pop(name)

    return logger


def group(items, size):
    rv = []
    i = 0
    while i < len(items):
        rv.append(items[i:i + size])
        i += size

    return rv


def next_power_of_two(num):
    rv = 1
    while rv < num:
        rv = rv << 1
    return rv


class Reducer(object):
    def __init__(self, target, **kwargs):
        self.target = target

        self.test_type = kwargs["test_types"][0]
        run_info = wpttest.get_run_info(kwargs["metadata_root"],
                                        kwargs["product"],
                                        debug=False)
        test_filter = wptrunner.TestFilter(include=kwargs["include"])
        self.test_loader = wptrunner.TestLoader(kwargs["tests_root"],
                                                kwargs["metadata_root"],
                                                [self.test_type],
                                                run_info,
                                                manifest_filer=test_filter)
        if kwargs["repeat"] == 1:
            logger.critical("Need to specify --repeat with more than one repetition")
            sys.exit(1)
        self.kwargs = kwargs

    def run(self):
        all_tests = self.get_initial_tests()

        tests = all_tests[:-1]
        target_test = [all_tests[-1]]

        if self.unstable(target_test):
            return target_test

        if not self.unstable(all_tests):
            return []

        chunk_size = next_power_of_two(int(len(tests) / 2))
        logger.debug("Using chunk size %i" % chunk_size)

        while chunk_size >= 1:
            logger.debug("%i tests remain" % len(tests))
            chunks = group(tests, chunk_size)
            chunk_results = [None] * len(chunks)

            for i, chunk in enumerate(chunks):
                logger.debug("Running chunk %i/%i of size %i" % (i + 1, len(chunks), chunk_size))
                trial_tests = []
                chunk_str = ""
                for j, inc_chunk in enumerate(chunks):
                    if i != j and chunk_results[j] in (None, False):
                        chunk_str += "+"
                        trial_tests.extend(inc_chunk)
                    else:
                        chunk_str += "-"
                logger.debug("Using chunks %s" % chunk_str)
                trial_tests.extend(target_test)

                chunk_results[i] = self.unstable(trial_tests)

                # if i == len(chunks) - 2 and all(item is False for item in chunk_results[:-1]):
                # Dangerous? optimisation that if you got stability for 0..N-1 chunks
                # it must be unstable with the Nth chunk
                #     chunk_results[i+1] = True
                #     continue

            new_tests = []
            keep_str = ""
            for result, chunk in zip(chunk_results, chunks):
                if not result:
                    keep_str += "+"
                    new_tests.extend(chunk)
                else:
                    keep_str += "-"

            logger.debug("Keeping chunks %s" % keep_str)

            tests = new_tests

            chunk_size = int(chunk_size / 2)

        return tests + target_test

    def unstable(self, tests):
        logger.debug("Running with %i tests" % len(tests))

        self.test_loader.tests = {self.test_type: tests}

        stdout, stderr = sys.stdout, sys.stderr
        sys.stdout = StringIO()
        sys.stderr = StringIO()

        with tempfile.NamedTemporaryFile() as f:
            args = self.kwargs.copy()
            args["log_raw"] = [f]
            args["capture_stdio"] = False
            wptrunner.setup_logging(args, {})
            wptrunner.run_tests(test_loader=self.test_loader, **args)
            wptrunner.logger.remove_handler(wptrunner.logger.handlers[0])
            is_unstable = self.log_is_unstable(f)

            sys.stdout, sys.stderr = stdout, stderr

        logger.debug("Result was unstable with chunk removed"
                     if is_unstable else "stable")

        return is_unstable

    def log_is_unstable(self, log_f):
        log_f.seek(0)

        statuses = defaultdict(set)

        def handle_status(item):
            if item["test"] == self.target:
                statuses[item["subtest"]].add(item["status"])

        def handle_end(item):
            if item["test"] == self.target:
                statuses[None].add(item["status"])

        reader.each_log(reader.read(log_f),
                        {"test_status": handle_status,
                         "test_end": handle_end})

        logger.debug(str(statuses))

        if not statuses:
            logger.error("Didn't get any useful output from wptrunner")
            log_f.seek(0)
            for item in reader.read(log_f):
                logger.debug(item)
            return None

        return any(len(item) > 1 for item in statuses.itervalues())

    def get_initial_tests(self):
        # Need to pass in arguments

        all_tests = self.test_loader.tests[self.test_type]
        tests = []
        for item in all_tests:
            tests.append(item)
            if item.url == self.target:
                break

        logger.debug("Starting with tests: %s" % ("\n".join(item.id for item in tests)))

        return tests


def do_reduce(**kwargs):
    target = kwargs.pop("target")
    reducer = Reducer(target, **kwargs)

    unstable_set = reducer.run()
    return unstable_set
