# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

import json
import os

from ..reader import LogHandler


class ResourceHandler(LogHandler):
    """Handler class for recording a resource usage profile."""

    def __init__(self, command_context, **kwargs):
        super().__init__(**kwargs)

        from mozbuild.util import construct_log_filename
        from mozsystemmonitor.resourcemonitor import SystemResourceMonitor

        # Get command name for log subdirectory
        handler = getattr(command_context, "handler", None)
        command_name = handler.name if handler else "test"
        log_subdir = os.path.join("logs", command_name)

        # Ensure log directory exists and create timestamped profile filename
        command_context._ensure_state_subdir_exists(log_subdir)
        self.build_resources_profile_path = command_context._get_state_filename(
            construct_log_filename("profile"), subdir=log_subdir
        )
        self.resources = SystemResourceMonitor(
            poll_interval=0.1,
        )
        self.resources.start()

    def shutdown(self, data):
        if not self.resources:
            return

        self.resources.stop()
        with open(
            self.build_resources_profile_path, "w", encoding="utf-8", newline="\n"
        ) as fh:
            to_write = json.dumps(self.resources.as_profile(), separators=(",", ":"))
            fh.write(to_write)

    def suite_start(self, data):
        self.current_suite = data.get("name")
        self.resources.begin_marker("suite", self.current_suite)

    def suite_end(self, data):
        self.resources.end_marker("suite", self.current_suite)

    def group_start(self, data):
        self.resources.begin_marker("test", data["name"])

    def group_end(self, data):
        self.resources.end_marker("test", data["name"])

    def test_start(self, data):
        self.resources.begin_test(data)

    def test_end(self, data):
        self.resources.end_test(data)

    def test_status(self, data):
        self.resources.test_status(data)

    def log(self, data):
        self.resources.test_status(data)

    def process_output(self, data):
        self.resources.test_status(data)

    def crash(self, data):
        self.resources.crash(data)
