from __future__ import print_function
import argparse
import os
import sys

from mozlog import structuredlog
from mozlog.handlers import BaseHandler, StreamHandler
from mozlog.formatters import MachFormatter
from six import iteritems
from six.moves.configparser import ConfigParser
from wptrunner import wptcommandline, wptrunner

here = os.path.abspath(os.path.dirname(__file__))

def setup_wptrunner_logging(logger):
    structuredlog.set_default_logger(logger)
    wptrunner.logger = logger
    wptrunner.wptlogging.setup_stdlib_logger()

class ResultHandler(BaseHandler):
    def __init__(self, verbose=False, logger=None):
        self.inner = StreamHandler(sys.stdout, MachFormatter())
        BaseHandler.__init__(self, self.inner)
        self.product = None
        self.verbose = verbose
        self.logger = logger

        self.register_message_handlers("wptrunner-test", {"set-product": self.set_product})

    def set_product(self, product):
        self.product = product

    def __call__(self, data):
        if self.product is not None and data["action"] in ["suite_start", "suite_end"]:
            # Hack: mozlog sets some internal state to prevent multiple suite_start or
            # suite_end messages. We actually want that here (one from the metaharness
            # and one from the individual test type harness), so override that internal
            # state (a better solution might be to not share loggers, but this works well
            # enough)
            self.logger._state.suite_started = True
            return

        if (not self.verbose and
            (data["action"] == "process_output" or
             data["action"] == "log" and data["level"] not in ["error", "critical"])):
            return

        if "test" in data:
            data = data.copy()
            data["test"] = "%s: %s" % (self.product, data["test"])

        return self.inner(data)

def test_settings():
    return {
        "include": "_test",
        "manifest-update": "",
        "no-capture-stdio": ""
    }

def read_config():
    parser = ConfigParser()
    parser.read("test.cfg")

    rv = {"general":{},
          "products":{}}

    rv["general"].update(dict(parser.items("general")))

    # This only allows one product per whatever for now
    for product in parser.sections():
        if product != "general":
            rv["products"][product] = {}
            for key, value in parser.items(product):
                rv["products"][product][key] = value

    return rv

def run_tests(product, kwargs):
    kwargs["test_paths"]["/_test/"] = {"tests_path": os.path.join(here, "testdata"),
                                       "metadata_path": os.path.join(here, "metadata")}

    wptrunner.run_tests(**kwargs)

def settings_to_argv(settings):
    rv = []
    for name, value in iteritems(settings):
        key = "--%s" % name
        if not value:
            rv.append(key)
        elif isinstance(value, list):
            for item in value:
                rv.extend([key, item])
        else:
            rv.extend([key, value])
    return rv

def set_from_args(settings, args):
    if args.test:
        settings["include"] = args.test
    if args.tags:
        settings["tags"] = args.tags

def run(config, args):
    logger = structuredlog.StructuredLogger("web-platform-tests")
    logger.add_handler(ResultHandler(logger=logger, verbose=args.verbose))
    setup_wptrunner_logging(logger)

    parser = wptcommandline.create_parser()

    logger.suite_start(tests=[])

    for product, product_settings in iteritems(config["products"]):
        if args.product and product not in args.product:
            continue

        settings = test_settings()
        settings.update(config["general"])
        settings.update(product_settings)
        settings["product"] = product
        set_from_args(settings, args)

        kwargs = vars(parser.parse_args(settings_to_argv(settings)))
        wptcommandline.check_args(kwargs)

        logger.send_message("wptrunner-test", "set-product", product)

        run_tests(product, kwargs)

    logger.send_message("wptrunner-test", "set-product", None)
    logger.suite_end()

def get_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument("-v", "--verbose", action="store_true", default=False,
                        help="verbose log output")
    parser.add_argument("--product", action="append",
                        help="Specific product to include in test run")
    parser.add_argument("--pdb", action="store_true",
                        help="Invoke pdb on uncaught exception")
    parser.add_argument("--tag", action="append", dest="tags",
                        help="tags to select tests")
    parser.add_argument("test", nargs="*",
                        help="Specific tests to include in test run")
    return parser

def main():
    config = read_config()

    args = get_parser().parse_args()

    try:
        run(config, args)
    except Exception:
        if args.pdb:
            import pdb
            import traceback
            print(traceback.format_exc())
            pdb.post_mortem()
        else:
            raise

if __name__ == "__main__":
    main()
