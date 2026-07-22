# Servo Unscientif Benchmark

This is a tool to run servo with webdriver enabled in a couple of analysis programs.
We currently support heaptrack and perf.
We automatically run a couple of webdriver based tests in in servo and allow you to record the data.
This is good if you need to compare the performance impact of changes you did.

These tests will have a lot of day to day variance in their performance, so you should not use this
for long term performance analysis.
The webdriver tests are currently very basic but should give you an idea on how to write them.
