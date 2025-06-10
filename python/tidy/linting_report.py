# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import json

#   annotation = {
#       "path": primary_span["file_name"],
#       "start_line": primary_span["line_start"],
#       "end_line": primary_span["line_end"],
#       "annotation_level": annotation_level,
#       "title": message.get("message", ""),
#       "message": message.get("rendered", ""),
#   }


class LintingReportManager:
    report = []

    def __init__(self, output):
        self.output = output

    def append(self, data):
        current_report = {
            "path": data[0],
            "start_line": data[1],
            "end_line": data[1],
            "annotation_level": "error",
            "title": data[2],
            "message": data[2],
        }
        self.report.append(current_report)

    def save(self):
        with open(self.output, "w", encoding="utf-8") as file:
            json.dump(self.report, file, indent=2)
            file.write("\n")
