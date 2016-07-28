#!/bin/sh
REPEAT_COUNT=100

for test_type in wpt css; do
    while read test_name; do
        echo "  - Checking $test_name"
        ./mach test-${test_type} --release --repeat $REPEAT_COUNT "$test_name" > tmp_log.txt
        if [ $? != 0 ]; then
            cat tmp_log.txt
            exit $?
        fi
    done < etc/ci/former_intermittents_${test_type}.txt
done

