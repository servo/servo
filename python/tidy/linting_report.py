# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import json
import os


class LintingReportManager:
    report = []
    count = 0

    def __init__(self, output):
        self.output = output

    def append(self, data):
        if len(self.report) >= 2:
            return

        current_report = {
            "path": data[0].removeprefix("./"),
            "start_line": data[1],
            "end_line": data[1],
            "annotation_level": "failure",
            "title": f"Mach test-tidy: {data[2]}",
            "message": data[2],
        }
        self.report.append(current_report)

    def annotation_log(self, severity, data):
        if self.count >= 10:
            return

        file_path = data[0].removeprefix("./")
        line_number = data[1]
        title = f"Mach test-tidy: {data[2]}"
        message = data[2]

        print(f"::{severity} file={file_path},line={line_number},endLine={line_number},title={title}::{message}")
        self.count += 1

    def combine_with_clippy(self, source):
        if not os.path.exists(source):
            return

        with open(source, "r", encoding="utf-8") as file:
            clippy_report = json.load(file)
            self.report.extend(clippy_report)

    def save(self):
        with open(self.output, "w", encoding="utf-8") as file:
            json.dump(self.report, file, indent=2)
            file.write("\n")
