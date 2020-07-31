# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import multiprocessing
import os
import sys
import mozlog
import grouping_formatter

here = os.path.split(__file__)[0]
servo_root = os.path.abspath(os.path.join(here, "..", ".."))


def wpt_path(*args):
    return os.path.join(here, *args)


def servo_path(*args):
    return os.path.join(servo_root, *args)


paths = {"include_manifest": wpt_path("include.ini"),
         "config": wpt_path("config.ini"),
         "ca-cert-path": wpt_path("web-platform-tests/tools/certs/cacert.pem"),
         "host-key-path": wpt_path("web-platform-tests/tools/certs/web-platform.test.key"),
         "host-cert-path": wpt_path("web-platform-tests/tools/certs/web-platform.test.pem")}
# Imports
sys.path.append(wpt_path("web-platform-tests", "tools"))
import localpaths  # noqa: F401,E402
from wptrunner import wptrunner, wptcommandline  # noqa: E402


def run_tests(**kwargs):
    set_defaults(kwargs)

    mozlog.commandline.log_formatters["servo"] = \
        (grouping_formatter.ServoFormatter, "Servo's grouping output formatter")
    mozlog.commandline.log_formatters["servojson"] = \
        (grouping_formatter.ServoJsonFormatter, "Servo's JSON logger of unexpected results")

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


def set_defaults(kwargs):
    if kwargs["product"] is None:
        kwargs["product"] = "servo"

    if kwargs["config"] is None and "config" in paths:
        kwargs["config"] = paths["config"]

    if kwargs["include_manifest"] is None and "include_manifest" in paths:
        kwargs["include_manifest"] = paths["include_manifest"]

    if kwargs["manifest_update"] is None:
        kwargs["manifest_update"] = False

    if kwargs["binary"] is None:
        bin_dir = "release" if kwargs["release"] else "debug"
        bin_name = "servo"
        if sys.platform == "win32":
            bin_name += ".exe"
        if "CARGO_TARGET_DIR" in os.environ:
            base_path = os.environ["CARGO_TARGET_DIR"]
        else:
            base_path = servo_path("target")

        target = kwargs.pop('target')
        if target:
            base_path = os.path.join(base_path, target)

        bin_path = os.path.join(base_path, bin_dir, bin_name)

        kwargs["binary"] = bin_path
        kwargs["webdriver_binary"] = bin_path

    if kwargs["processes"] is None:
        kwargs["processes"] = multiprocessing.cpu_count()

    if kwargs["ca_cert_path"] is None:
        kwargs["ca_cert_path"] = paths["ca-cert-path"]

    if kwargs["host_key_path"] is None:
        kwargs["host_key_path"] = paths["host-key-path"]

    if kwargs["host_cert_path"] is None:
        kwargs["host_cert_path"] = paths["host-cert-path"]

    if kwargs["ssl_type"] is None:
        kwargs["ssl_type"] = "pregenerated"

    kwargs["user_stylesheets"].append(servo_path("resources", "ahem.css"))

    wptcommandline.check_args(kwargs)

    if kwargs.pop("layout_2020"):
        kwargs["test_paths"]["/"]["metadata_path"] = wpt_path("metadata-layout-2020")
        kwargs["test_paths"]["/_mozilla/"]["metadata_path"] = wpt_path("mozilla/meta-layout-2020")
        kwargs["include_manifest"] = wpt_path("include-layout-2020.ini")


def main():
    parser = wptcommandline.create_parser()
    kwargs = vars(parser.parse_args())
    return run_tests(**kwargs)


if __name__ == "__main__":
    sys.exit(0 if main() else 1)
