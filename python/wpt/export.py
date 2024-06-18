#!/usr/bin/env python

# Copyright 2023 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# pylint: disable=missing-docstring
# pylint: disable=invalid-name

import json
import logging
import os
import sys

from exporter import WPTSync


def main() -> int:
    context = json.loads(os.environ['GITHUB_CONTEXT'])
    logging.getLogger().level = logging.INFO

    success = WPTSync(
        servo_repo='servo/servo',
        wpt_repo='web-platform-tests/wpt',
        downstream_wpt_repo='servo-wpt-sync/web-platform-tests',
        servo_path='./servo',
        wpt_path='./wpt',
        github_api_token=os.environ['WPT_SYNC_TOKEN'],
        github_api_url='https://api.github.com/',
        github_username='servo-wpt-sync',
        github_email='ghbot+wpt-sync@servo.org',
        github_name='Servo WPT Sync',
    ).run(context["event"])
    return 0 if success else 1


if __name__ == "__main__":
    sys.exit(main())
