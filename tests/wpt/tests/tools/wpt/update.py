# mypy: allow-untyped-defs

import os
import sys

from mozlog import commandline

wpt_root = os.path.abspath(os.path.join(os.path.dirname(__file__), os.pardir, os.pardir))
sys.path.insert(0, os.path.abspath(os.path.join(wpt_root, "tools")))


def manifest_update(test_paths):
    from manifest import manifest  # type: ignore
    for url_base, paths in test_paths.items():
        manifest.load_and_update(
            paths["tests_path"],
            paths["manifest_path"],
            url_base)


def create_parser_update():
    from wptrunner import wptcommandline

    return wptcommandline.create_parser_metadata_update()


def update_expectations(_, **kwargs):
    from wptrunner import metadata, wptcommandline

    commandline.setup_logging("web-platform-tests",
                              kwargs,
                              {"mach": sys.stdout},
                              formatter_defaults=None)

    if not kwargs["tests_root"]:
        kwargs["tests_root"] = wpt_root

    # This matches the manifest path we end up using in `wpt run`
    if not kwargs["manifest_path"]:
        kwargs["manifest_path"] = os.path.join(wpt_root, "MANIFEST.json")

    kwargs = wptcommandline.check_args_metadata_update(kwargs)

    update_properties = metadata.get_properties(properties_file=kwargs["properties_file"],
                                                extra_properties=kwargs["extra_property"],
                                                config=kwargs["config"],
                                                product=kwargs["product"])

    manifest_update(kwargs["test_paths"])
    metadata.update_expected(kwargs["test_paths"],
                             kwargs["run_log"],
                             update_properties=update_properties,
                             full_update=False,
                             disable_intermittent=kwargs["update_intermittent"],
                             update_intermittent=kwargs["update_intermittent"],
                             remove_intermittent=kwargs["update_intermittent"])
