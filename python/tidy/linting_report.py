# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.


class LintingReportManager:
    count = 0

    def __init__(self, limit):
        self.limit = limit

    def annotation_log(self, severity, data):
        if self.count >= self.limit:
            return

        file_path = data[0].removeprefix("./")
        line_number = data[1]
        title = f"Mach test-tidy: {data[2]}"
        message = data[2]

        print(
            f"::{severity} file={file_path},line={line_number},endLine={line_number},title={title}::{message}",
            flush=True,
        )
        self.count += 1
