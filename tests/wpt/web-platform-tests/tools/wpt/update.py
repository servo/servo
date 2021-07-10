import os
import sys

wpt_root = os.path.abspath(os.path.join(os.path.dirname(__file__), os.pardir, os.pardir))
sys.path.insert(0, os.path.abspath(os.path.join(wpt_root, "tools")))


def create_parser_update():
    from wptrunner import wptcommandline

    return wptcommandline.create_parser_update()


def update_expectations(venv, **kwargs):
    from wptrunner import wptcommandline
    from wptrunner.update import setup_logging, WPTUpdate

    if not kwargs["tests_root"]:
        kwargs["tests_root"] = wpt_root

    if not kwargs["manifest_path"]:
        kwargs["manifest_path"] = os.path.join(wpt_root, "MANIFEST.json")

    if "product" not in kwargs["extra_property"]:
        kwargs["extra_property"].append("product")

    wptcommandline.check_args_update(kwargs)

    logger = setup_logging(kwargs, {"mach": sys.stdout})

    updater = WPTUpdate(logger, **kwargs)
    updater.run()
