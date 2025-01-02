This directory contains tests for the
[Compute Pressure](https://w3c.github.io/compute-pressure/) specification.

## How to write tests
### Tests that only need to run on a window
To test this API, one needs to be able to control the pressure data that will
be reported to script. At a high level, this is done by calling certain
[WebDriver endpoints](https://w3c.github.io/compute-pressure/#automation) via
their corresponding
[testdriver](https://web-platform-tests.org/writing-tests/testdriver.html#compute-pressure)
wrappers.

### Tests that need to run on windows and dedicated workers
Certain [testdriver
limitations](https://web-platform-tests.org/writing-tests/testdriver.html#using-test-driver-in-other-browsing-contexts)
require calls to be made from the top-level test context, which effectively
prevents us from simply [running the same test from multiple globals with
any.js](https://web-platform-tests.org/writing-tests/testharness.html#tests-for-other-or-multiple-globals-any-js).

What we do instead is [write all tests for the Window
global](https://web-platform-tests.org/writing-tests/testharness.html#window-tests),
use
[variants](https://web-platform-tests.org/writing-tests/testharness.html#specifying-test-variants)
for specifying different globals and using the `pressure_test()` and
`mark_as_done()` helpers.

In short, the boilerplate for a new test `foo.https.window.js` looks like this:

``` js
// META: variant=?globalScope=window
// META: variant=?globalScope=dedicated_worker
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=./resources/common.js

pressure_test(async t => {
}, 'my test');

mark_as_done();
```

- The variants specify which global context the tests should run on. The only
  two options are `window` and `dedicated_worker`.
- We need to include all those scripts for the testdriver and
  [RemoteContext](../common/dispatcher/README.md) infrastructure to work.
- `pressure_test()` is a wrapper around a `promise_test()` that takes care of
  running the test either in the current context (when `globalScope=window`) or
  in a dedicated worker via `RemoteContext` and `fetch_tests_from_worker()`
  (when `globalScope=dedicated_worker`).
- `mark_as_done()` is a no-op when `globalScope=window`, but is necessary when
  `globalScope=dedicated_worker` to ensure that all tests have run and that
  [`done()`](https://web-platform-tests.org/writing-tests/testharness-api.html#Test.done)
  is called in the worker context.

### Shared workers
Since custom pressure states are stored in a top-level navigables, they are
currently not integrated with shared workers (see [spec issue
285](https://github.com/w3c/compute-pressure/issues/285)) and support for
testing shared workers is limited.
