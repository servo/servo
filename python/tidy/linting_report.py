# Copyright 2025 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from dataclasses import dataclass
from typing import Any, Literal, NotRequired


@dataclass
class GithubAnnotation:
    file_name: str
    line_start: int
    line_end: int
    level: Literal["notice", "warning", "error"]
    title: str
    message: str
    column_start: NotRequired[int]
    column_end: NotRequired[int]


class GitHubAnnotationManager:
    def __init__(self, annotation_prefix: str, limit: int = 10):
        self.annotation_prefix: str = annotation_prefix
        self.limit: int = limit
        self.severenty_map: dict[str, Literal["notice", "warning", "error"]] = {
            "help": "notice",
            "note": "notice",
            "warning": "warning",
            "error": "error",
        }
        self.total_count: int = 0

    def clean_path(self, path: str):
        return path.removeprefix("./")

    def escape(self, s: str):
        return s.replace("\r", "%0D").replace("\n", "%0A")

    def emit_annotation(
        self,
        title: str,
        message: str,
        file_name: str,
        line_start: int,
        line_end: int = None,
        annotation_level: str = None,
        column_start: int = None,
        column_end: int = None,
    ):
        if self.total_count >= self.limit:
            return

        if annotation_level is None:
            annotation_level = "error"

        if line_end is None:
            line_end = line_start

        annotation: GithubAnnotation = {
            "title": f"{self.annotation_prefix}: {title}",
            "message": self.escape(message),
            "file_name": self.clean_path(file_name),
            "line_start": line_start,
            "line_end": line_end,
            "level": annotation_level,
        }

        if line_start == line_end and column_start is not None and column_end is not None:
            annotation["column_start"] = column_start
            annotation["column_end"] = column_end

        line_info = f"line={annotation['line_start']},endLine={annotation['line_end']},title={annotation['title']}"

        column_info = ""
        if "column_end" in annotation and "column_start" in annotation:
            column_info = f"col={annotation['column_start']},endColumn={annotation['column_end']},"

        print(
            f"::{annotation['level']} file={annotation['file_name']},{column_info}{line_info}::{annotation['message']}"
        )

        self.total_count += 1

    def emit_annotations_for_clippy(self, data: list[dict[str, Any]]):
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

            self.emit_annotation(
                title,
                rendered_message,
                primary_span["file_name"],
                primary_span["line_start"],
                primary_span["line_end"],
                annotation_level,
                primary_span["column_start"],
                primary_span["column_end"],
            )
