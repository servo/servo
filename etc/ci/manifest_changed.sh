#!/bin/bash
diff=$(git diff -- tests/**/MANIFEST.json)
echo "$diff"
[[ ! $diff ]]
