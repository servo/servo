#!/usr/bin/env bash

# Usage: ./clippy-to-annotations.sh < input.json > output.json
# Will exit with an error if there are no valid annotations.

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
            else "failure"
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

if [[ "$output" == "[]" || -z "$output" ]]; then
  echo "❌ No annotations to output." >&2
  exit 0
fi

# Output the final result
echo "$output"
