# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import os
import sys

import grouping_formatter
import mozlog
import multiprocessing

SCRIPT_PATH = os.path.abspath(os.path.dirname(__file__))
SERVO_ROOT = os.path.abspath(os.path.join(SCRIPT_PATH, "..", ".."))
WPT_TOOLS_PATH = os.path.join(SCRIPT_PATH, "web-platform-tests", "tools")
CERTS_PATH = os.path.join(WPT_TOOLS_PATH, "certs")

sys.path.insert(0, WPT_TOOLS_PATH)
import update  # noqa: F401,E402
import localpaths  # noqa: F401,E402


def determine_build_type(kwargs: dict, target_dir: str):
    if kwargs["release"]:
        return "release"
    elif kwargs["debug"]:
        return "debug"
    elif os.path.exists(os.path.join(target_dir, "debug")):
        return "debug"
    elif os.path.exists(os.path.join(target_dir, "release")):
        return "release"
    return "debug"


def set_if_none(args: dict, key: str, value):
    if key not in args or args[key] is None:
        args[key] = value


def update_args_for_layout_2020(kwargs: dict):
    if kwargs.pop("layout_2020"):
        kwargs["test_paths"]["/"]["metadata_path"] = os.path.join(
            SCRIPT_PATH, "metadata-layout-2020"
        )
        kwargs["test_paths"]["/_mozilla/"]["metadata_path"] = os.path.join(
            SCRIPT_PATH, "mozilla", "meta-layout-2020"
        )
        kwargs["include_manifest"] = os.path.join(
            SCRIPT_PATH, "include-layout-2020.ini"
        )


def run_tests(**kwargs):
    from wptrunner import wptrunner
    from wptrunner import wptcommandline

    # By default, Rayon selects the number of worker threads based on the
    # available CPU count. This doesn't work very well when running tests on CI,
    # since we run so many Servo processes in parallel. The result is a lot of
    # extra timeouts. Instead, force Rayon to assume we are running on a 2 CPU
    # environment.
    os.environ["RAYON_RS_NUM_CPUS"] = "2"
    os.environ["RUST_BACKTRACE"] = "1"
    os.environ["HOST_FILE"] = os.path.join(SERVO_ROOT, "tests", "wpt", "hosts")

    set_if_none(kwargs, "product", "servo")
    set_if_none(kwargs, "config", os.path.join(SCRIPT_PATH, "config.ini"))
    set_if_none(kwargs, "include_manifest", os.path.join(SCRIPT_PATH, "include.ini"))
    set_if_none(kwargs, "manifest_update", False)
    set_if_none(kwargs, "processes", multiprocessing.cpu_count())

    set_if_none(kwargs, "ca_cert_path", os.path.join(CERTS_PATH, "cacert.pem"))
    set_if_none(
        kwargs, "host_key_path", os.path.join(CERTS_PATH, "web-platform.test.key")
    )
    set_if_none(
        kwargs, "host_cert_path", os.path.join(CERTS_PATH, "web-platform.test.pem")
    )

    kwargs["user_stylesheets"].append(os.path.join(SERVO_ROOT, "resources", "ahem.css"))

    if "CARGO_TARGET_DIR" in os.environ:
        target_dir = os.path.join(os.environ["CARGO_TARGET_DIR"])
    else:
        target_dir = os.path.join(SERVO_ROOT, "target")
    default_binary_path = os.path.join(
        target_dir, determine_build_type(kwargs, target_dir), "servo"
    )
    if sys.platform == "win32":
        target_dir += ".exe"

    set_if_none(kwargs, "binary", default_binary_path)
    set_if_none(kwargs, "webdriver_binary", default_binary_path)

    if kwargs.pop("rr_chaos"):
        kwargs["debugger"] = "rr"
        kwargs["debugger_args"] = "record --chaos"
        kwargs["repeat_until_unexpected"] = True
        # TODO: Delete rr traces from green test runs?

    prefs = kwargs.pop("prefs")
    if prefs:
        kwargs["binary_args"] = ["--pref=" + pref for pref in prefs]

    if not kwargs.get("no_default_test_types"):
        test_types = {
            "servo": ["testharness", "reftest", "wdspec"],
            "servodriver": ["testharness", "reftest"],
        }
        product = kwargs.get("product") or "servo"
        kwargs["test_types"] = test_types[product]

    wptcommandline.check_args(kwargs)
    update_args_for_layout_2020(kwargs)

    mozlog.commandline.log_formatters["servo"] = (
        grouping_formatter.ServoFormatter,
        "Servo's grouping output formatter",
    )
    mozlog.commandline.log_formatters["servojson"] = (
        grouping_formatter.ServoJsonFormatter,
        "Servo's JSON logger of unexpected results",
    )

    use_mach_logging = False
    if len(kwargs["test_list"]) == 1:
        file_ext = os.path.splitext(kwargs["test_list"][0])[1].lower()
        if file_ext in [".htm", ".html", ".js", ".xhtml", ".xht", ".py"]:
            use_mach_logging = True

    if use_mach_logging:
        wptrunner.setup_logging(kwargs, {"mach": sys.stdout})
    else:
        wptrunner.setup_logging(kwargs, {"servo": sys.stdout})

    success = wptrunner.run_tests(**kwargs)
    return 0 if success else 1


def update_tests(**kwargs):
    from update import updatecommandline

    set_if_none(kwargs, "product", "servo")
    set_if_none(kwargs, "config", os.path.join(SCRIPT_PATH, "config.ini"))
    kwargs["store_state"] = False

    updatecommandline.check_args(kwargs)

    logger = update.setup_logging(kwargs, {"mach": sys.stdout})
    return_value = update.run_update(logger, **kwargs)
    return 1 if return_value is update.exit_unclean else 0


def main():
    from wptrunner import wptcommandline

    parser = wptcommandline.create_parser()
    kwargs = vars(parser.parse_args())
    return run_tests(**kwargs)


if __name__ == "__main__":
    sys.exit(0 if main() else 1)
