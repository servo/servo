# Copyright 2025 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.


from typing import List, Literal, NotRequired, TypedDict

import colorama


class GithubAnnotation(TypedDict):
    file_name: str
    line_start: int
    line_end: int
    level: Literal["notice", "warning", "error"]
    title: str
    message: str
    column_start: NotRequired[int]
    column_end: NotRequired[int]


class LintingReportManager:
    def __init__(self, annotation_prefix: str, limit: int):
        self.annotation_prefix: str = annotation_prefix
        self.limit: int = limit
        self.severenty_map: dict[str, Literal["notice", "warning", "error"]] = {
            "help": "notice",
            "note": "notice",
            "warning": "warning",
            "error": "error",
        }
        self.annotations: List[GithubAnnotation] = []
        self.total_count = 0
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

    def append_annotation(
        self,
        title,
        message,
        file_name,
        line_start,
        line_end=None,
        annotation_level=None,
        column_start=None,
        column_end=None,
    ):
        if self.total_count >= self.limit:
            return

        if annotation_level is None:
            annotation_level = "error"

        if line_end is None:
            line_end = line_start

        annotation: GithubAnnotation = {
            "title": f"./Mach {self.annotation_prefix}: {title}",
            "message": self.escape(message),
            "file_name": self.clean_path(file_name),
            "line_start": line_start,
            "line_end": line_end,
            "level": annotation_level,
        }

        if line_start == line_end and column_start is not None and column_end is not None:
            annotation["column_start"] = column_start
            annotation["column_end"] = column_end

        self.annotations.append(annotation)
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

            self.append_annotation(
                title,
                rendered_message,
                primary_span["file_name"],
                primary_span["line_start"],
                primary_span["line_end"],
                annotation_level,
                primary_span["column_start"],
                primary_span["column_end"],
            )

    def emit_github_annotations(self):
        for annotation in self.annotations:
            self.emit_github_annotation(annotation)

    def emit_github_annotation(self, annotation: GithubAnnotation):
        line_info = f"line={annotation['line_start']},endLine={annotation['line_end']},title={annotation['title']}"

        column_info = ""
        if "column_end" in annotation and "column_start" in annotation:
            column_info = f"col={annotation['column_start']},endColumn={annotation['column_end']},"

        print(
            (
                f"::{annotation['level']} file={annotation['file_name']},"
                f"{column_info}{line_info}"
                f"::{annotation['message']}"
            ),
            flush=True,
        )
