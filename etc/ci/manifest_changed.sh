#!/bin/bash
diff=$(git diff -- tests/wpt/**/MANIFEST.json)
echo "$diff"
[[ ! $diff ]]
