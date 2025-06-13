# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.


from typing import List, Literal, TypedDict

import colorama


class RequiredAnnotation(TypedDict):
    file_name: str
    line_start: int
    line_end: int
    level: Literal["notice", "warning", "error"]
    title: str
    message: str


class OptionalAnnotation(RequiredAnnotation, total=False):
    column_start: int
    column_end: int


class LintingReportManager:
    def __init__(self, limit: int):
        self.limit: int = limit
        self.severenty_map: dict[str, Literal["notice", "warning", "error"]] = {
            "help": "notice",
            "note": "notice",
            "warning": "warning",
            "error": "error",
        }
        self.logs: List[OptionalAnnotation] = []
        self.total_count = 0
        self.error_count = 0
        colorama.init()

    def error_log(self, error):
        print(
            "\r  | "
            + f"{colorama.Fore.BLUE}{error[0]}{colorama.Style.RESET_ALL}:"
            + f"{colorama.Fore.YELLOW}{error[1]}{colorama.Style.RESET_ALL}: "
            + f"{colorama.Fore.RED}{error[2]}{colorama.Style.RESET_ALL}"
        )

    def clean_path(self, path):
        return path.removeprefix("./")

    def escape(self, s):
        return s.replace("\r", "%0D").replace("\n", "%0A")

    def append_log(self, annotation: OptionalAnnotation):
        if self.total_count >= self.limit:
            return
        self.logs.append(annotation)
        self.total_count += 1

    def filter_clippy_log(self, data):
        for item in data:
            if self.total_count >= self.limit:
                break

            message = item.get("message")
            if not message:
                continue

            spans = message.get("spans") or []
            primary_span = next((s for s in spans if s.get("is_primary")), None)
            if not primary_span:
                continue

            annotation_level = self.severenty_map.get(message.get("level"), "error")
            title = self.escape(message.get("message", ""))
            rendered_message = self.escape(message.get("rendered", ""))

            annotation: OptionalAnnotation = {
                "title": f"Mach clippy: {title}",
                "message": rendered_message,
                "file_name": primary_span["file_name"],
                "line_start": primary_span["line_start"],
                "line_end": primary_span["line_end"],
                "column_start": primary_span["column_start"],
                "column_end": primary_span["column_end"],
                "level": annotation_level,
            }

            if primary_span.get("line_start") == primary_span.get("line_end"):
                annotation["column_start"] = primary_span["column_start"]
                annotation["column_end"] = primary_span["column_end"]

            self.logs.append(annotation)
            self.total_count += 1

    def logs_annotation(self):
        for annotation in self.logs:
            self.log_annotation(annotation)

    def log_annotation(self, annotation: OptionalAnnotation):
        if "column_start" in annotation and "column_end" in annotation:
            print(
                (
                    f"::{annotation['level']} file={annotation['file_name']},"
                    f"line={annotation['line_start']},"
                    f"endLine={annotation['line_end']},"
                    f"col={annotation['column_start']},"
                    f"endColumn={annotation['column_end']},"
                    f"title={annotation['title']}::"
                    f"{annotation['message']}"
                ),
                flush=True,
            )
        else:
            print(
                (
                    f"::{annotation['level']} file={annotation['file_name']},"
                    f"line={annotation['line_start']},"
                    f"endLine={annotation['line_end']},"
                    f"title={annotation['title']}::"
                    f"{annotation['message']}"
                ),
                flush=True,
            )
