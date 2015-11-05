#!/bin/bash
diff=$(git diff -- */*/Cargo.lock)
echo "$diff"
[[ ! $diff ]]
