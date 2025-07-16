# Copyright 2025 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from dataclasses import dataclass
from typing import Any, Literal, Optional


@dataclass
class GithubAnnotation:
    file_name: str
    line_start: int
    line_end: int
    level: Literal["notice", "warning", "error"]
    title: str
    message: str
    column_start: Optional[int] = None
    column_end: Optional[int] = None


class GitHubAnnotationManager:
    def __init__(self, annotation_prefix: str, limit: int = 10) -> None:
        self.annotation_prefix: str = annotation_prefix
        self.limit: int = limit
        self.total_count: int = 0

    def clean_path(self, path: str) -> str:
        return path.removeprefix("./")

    def escape(self, s: str) -> str:
        return s.replace("\r", "%0D").replace("\n", "%0A")

    def emit_annotation(
        self,
        title: str,
        message: str,
        file_name: str,
        line_start: int,
        line_end: int | None = None,
        annotation_level: Literal["notice", "warning", "error"] = "error",
        column_start: int | None = None,
        column_end: int | None = None,
    ) -> None:
        if self.total_count >= self.limit:
            return

        if line_end is None:
            line_end = line_start

        annotation = GithubAnnotation(
            title=f"{self.annotation_prefix}: {self.escape(title)}",
            message=self.escape(message),
            file_name=self.clean_path(file_name),
            line_start=line_start,
            line_end=line_end,
            level=annotation_level,
            column_start=column_start,
            column_end=column_end,
        )

        line_info = f"line={annotation.line_start},endLine={annotation.line_end},title={annotation.title}"

        column_info = ""
        if line_start == line_end and annotation.column_end is not None and annotation.column_start is not None:
            column_info = f"col={annotation.column_start},endColumn={annotation.column_end},"

        print(f"::{annotation.level} file={annotation.file_name},{column_info}{line_info}::{annotation.message}")

        self.total_count += 1

    def emit_annotations_for_clippy(self, data: list[dict[str, Any]]) -> None:
        severenty_map: dict[str, Literal["notice", "warning", "error"]] = {
            "help": "notice",
            "note": "notice",
            "warning": "warning",
            "error": "error",
        }

        for item in data:
            if self.total_count >= self.limit:
                break

            message = item.get("message")
            if not message:
                continue

            spans = message.get("spans") or []
            primary_span = next((span for span in spans if span.get("is_primary")), None)
            if not primary_span:
                continue

            annotation_level = severenty_map.get(message.get("level"), "error")
            title = message.get("message", "")
            rendered_message = message.get("rendered", "")

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
