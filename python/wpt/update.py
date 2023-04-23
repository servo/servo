# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
# pylint: disable=missing-docstring

import os

from . import WPT_PATH, update_args_for_layout_2020
from . import importer


def set_if_none(args: dict, key: str, value):
    if key not in args or args[key] is None:
        args[key] = value


def update_tests(**kwargs):
    set_if_none(kwargs, "product", "servo")
    set_if_none(kwargs, "config", os.path.join(WPT_PATH, "config.ini"))
    kwargs["store_state"] = False

    importer.check_args(kwargs)
    update_args_for_layout_2020(kwargs)

    return 1 if not importer.run_update(**kwargs) else 0


def create_parser(**kwargs):
    return importer.create_parser()
