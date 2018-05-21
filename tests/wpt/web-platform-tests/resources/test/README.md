# `testharness.js` test suite

The test suite for the `testharness.js` testing framework.

## Executing Tests

Install the following dependencies:

- [Python 2.7.9+](https://www.python.org/)
- [the tox Python package](https://tox.readthedocs.io/en/latest/)
- [the Mozilla Firefox web browser](https://mozilla.org/firefox)
- [the GeckoDriver server](https://github.com/mozilla/geckodriver)

Make sure `geckodriver` can be found in your `PATH`.

Currently, the tests should be run with the latest *Firefox Nightly*. In order to
specify the path to Firefox Nightly, use the following command-line option:

    tox -- --binary=/path/to/FirefoxNightly

### Automated Script

Alternatively, you may run `tools/ci/ci_resources_unittest.sh`, which only depends on
Python 2. The script will install other dependencies automatically and start `tox` with
the correct arguments.

## Authoring Tests

Test cases are expressed as `.html` files located within the `tests/`
sub-directory. Each test should include the `testharness.js` library with the
following markup:

    <script src="../../testharness.js"></script>
    <script src="../../testharnessreport.js"></script>

This should be followed by one or more `<script>` tags that interface with the
`testharness.js` API in some way. For example:

    <script>
    test(function() {
        1 = 1;
      }, 'This test is expected to fail.');
    </script>

Finally, each test may include a summary of the expected results as a JSON
string within a `<script>` tag with an `id` of `"expected"`, e.g.:

    <script type="text/json" id="expected">
    {
      "summarized_status": {
        "message": null,
        "stack": null,
        "status_string": "OK"
      },
      "summarized_tests": [
        {
          "message": "ReferenceError: invalid assignment left-hand side",
          "name": "Sample HTML5 API Tests",
          "properties": {},
          "stack": "(implementation-defined)",
          "status_string": "FAIL"
        }
      ],
      "type": "complete"
    }
    </script>

This is useful to test, for example, whether asserations that should fail or
throw actually do.
