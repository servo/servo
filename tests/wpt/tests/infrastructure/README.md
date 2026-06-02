This directory contains a number of tests to ensure test running
infrastructure is operating correctly:

 * The tests in assumptions/ are designed to test UA assumptions
   documented in [assumptions.md](/docs/writing-tests/assumptions.md).

 * The tests in server/ are designed to test the WPT server configuration

 * The tests in expected-fail/ should all fail.

To update the expectations stored in metadata/, you want to use the `wpt`
tool with an invocation such as `./wpt update-expectations --metadata
infrastructure/metadata --manifest MANIFEST.json [wptreport.json]` with one
or more wptreport.json files.
