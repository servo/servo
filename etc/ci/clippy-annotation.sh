#!/usr/bin/env bash

# Usage: ./mach clippy --message-format=json --use-crown --locked -- -- --deny warnings | ./etc/ci/clippy-annotation.sh > temp/clippy-output.json

set -euo pipefail

output=$(jq -c '
  . as $in
  | ($in.message.spans // [] | map(select(.is_primary == true)) | first) as $primarySpan
  | if $primarySpan == null then
      empty
    else
      {
        path: $primarySpan.file_name,
        start_line: $primarySpan.line_start,
        end_line: $primarySpan.line_end,
        annotation_level: (
          $in.message.level
          | if . == "help" or . == "note" then "notice"
            elif . == "warning" then "warning"
            else "error"
            end
        ),
        title: $in.message.message,
        message: $in.message.rendered
      }
      | if .start_line == .end_line then
          . + {
            start_column: $primarySpan.column_start,
            end_column: $primarySpan.column_end
          }
        else
          .
        end
    end
' | jq -s '.')

echo "$output"

if echo "$output" | jq -e 'map(select(.annotation_level == "error")) | length > 0' >/dev/null; then
  exit 1
fi
