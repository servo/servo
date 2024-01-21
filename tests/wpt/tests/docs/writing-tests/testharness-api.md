# testharness.js API

```eval_rst

.. contents:: Table of Contents
   :depth: 3
   :local:
   :backlinks: none
```

testharness.js provides a framework for writing testcases. It is intended to
provide a convenient API for making common assertions, and to work both
for testing synchronous and asynchronous DOM features in a way that
promotes clear, robust, tests.

## Markup ##

The test harness script can be used from HTML or SVG documents and workers.

From an HTML or SVG document, start by importing both `testharness.js` and
`testharnessreport.js` scripts into the document:

```html
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
```

Refer to the [Web Workers](#web-workers) section for details and an example on
testing within a web worker.

Within each file one may define one or more tests. Each test is atomic in the
sense that a single test has a single status (`PASS`/`FAIL`/`TIMEOUT`/`NOTRUN`).
Within each test one may have a number of asserts. The test fails at the first
failing assert, and the remainder of the test is (typically) not run.

**Note:** From the point of view of a test harness, each document
using testharness.js is a single "test" and each js-defined
[`Test`](#Test) is referred to as a "subtest".

By default tests must be created before the load event fires. For ways
to create tests after the load event, see [determining when all tests
are complete](#determining-when-all-tests-are-complete).

### Harness Timeout ###

Execution of tests on a page is subject to a global timeout. By
default this is 10s, but a test runner may set a timeout multiplier
which alters the value according to the requirements of the test
environment (e.g. to give a longer timeout for debug builds).

Long-running tests may opt into a longer timeout by providing a
`<meta>` element:

```html
<meta name="timeout" content="long">
```

By default this increases the timeout to 60s, again subject to the
timeout multiplier.

Tests which define a large number of subtests may need to use the
[variant](testharness.html#specifying-test-variants) feature to break
a single test document into several chunks that complete inside the
timeout.

Occasionally tests may have a race between the harness timing out and
a particular test failing; typically when the test waits for some
event that never occurs. In this case it is possible to use
[`Test.force_timeout()`](#Test.force_timeout) in place of
[`assert_unreached()`](#assert_unreached), to immediately fail the
test but with a status of `TIMEOUT`. This should only be used as a
last resort when it is not possible to make the test reliable in some
other way.

## Defining Tests ##

### Synchronous Tests ###

```eval_rst
.. js:autofunction:: <anonymous>~test
   :short-name:
```
A trivial test for the DOM [`hasFeature()`](https://dom.spec.whatwg.org/#dom-domimplementation-hasfeature)
method (which is defined to always return true) would be:

```js
test(function() {
  assert_true(document.implementation.hasFeature());
}, "hasFeature() with no arguments")
```

### Asynchronous Tests ###

Testing asynchronous features is somewhat more complex since the
result of a test may depend on one or more events or other
callbacks. The API provided for testing these features is intended to
be rather low-level but applicable to many situations.

```eval_rst
.. js:autofunction:: async_test

```

Create a [`Test`](#Test):

```js
var t = async_test("DOMContentLoaded")
```

Code is run as part of the test by calling the [`step`](#Test.step)
method with a function containing the test
[assertions](#assert-functions):

```js
document.addEventListener("DOMContentLoaded", function(e) {
  t.step(function() {
    assert_true(e.bubbles, "bubbles should be true");
  });
});
```

When all the steps are complete, the [`done`](#Test.done) method must
be called:

```js
t.done();
```

`async_test` can also takes a function as first argument.  This
function is called with the test object as both its `this` object and
first argument. The above example can be rewritten as:

```js
async_test(function(t) {
  document.addEventListener("DOMContentLoaded", function(e) {
    t.step(function() {
      assert_true(e.bubbles, "bubbles should be true");
    });
    t.done();
  });
}, "DOMContentLoaded");
```

In many cases it is convenient to run a step in response to an event or a
callback. A convenient method of doing this is through the `step_func` method
which returns a function that, when called runs a test step. For example:

```js
document.addEventListener("DOMContentLoaded", t.step_func(function(e) {
  assert_true(e.bubbles, "bubbles should be true");
  t.done();
}));
```

As a further convenience, the `step_func` that calls
[`done`](#Test.done) can instead use
[`step_func_done`](#Test.step_func_done), as follows:

```js
document.addEventListener("DOMContentLoaded", t.step_func_done(function(e) {
  assert_true(e.bubbles, "bubbles should be true");
}));
```

For asynchronous callbacks that should never execute,
[`unreached_func`](#Test.unreached_func) can be used. For example:

```js
document.documentElement.addEventListener("DOMContentLoaded",
  t.unreached_func("DOMContentLoaded should not be fired on the document element"));
```

**Note:** the `testharness.js` doesn't impose any scheduling on async
tests; they run whenever the step functions are invoked. This means
multiple tests in the same global can be running concurrently and must
take care not to interfere with each other.

### Promise Tests ###

```eval_rst
.. js:autofunction:: promise_test
```

`test_function` is a function that receives a new [Test](#Test) as an
argument. It must return a promise. The test completes when the
returned promise settles. The test fails if the returned promise
rejects.

E.g.:

```js
function foo() {
  return Promise.resolve("foo");
}

promise_test(function() {
  return foo()
    .then(function(result) {
      assert_equals(result, "foo", "foo should return 'foo'");
    });
}, "Simple example");
```

In the example above, `foo()` returns a Promise that resolves with the string
"foo". The `test_function` passed into `promise_test` invokes `foo` and attaches
a resolve reaction that verifies the returned value.

Note that in the promise chain constructed in `test_function`
assertions don't need to be wrapped in [`step`](#Test.step) or
[`step_func`](#Test.step_func) calls.

It is possible to mix promise tests with callback functions using
[`step`](#Test.step). However this tends to produce confusing tests;
it's recommended to convert any asynchronous behaviour into part of
the promise chain. For example, instead of

```js
promise_test(t => {
  return new Promise(resolve => {
    window.addEventListener("DOMContentLoaded", t.step_func(event => {
      assert_true(event.bubbles, "bubbles should be true");
      resolve();
    }));
  });
}, "DOMContentLoaded");
```

Try,

```js
promise_test(() => {
  return new Promise(resolve => {
    window.addEventListener("DOMContentLoaded", resolve);
  }).then(event => {
    assert_true(event.bubbles, "bubbles should be true");
  });
}, "DOMContentLoaded");
```

**Note:** Unlike asynchronous tests, testharness.js queues promise
tests so the next test won't start running until after the previous
promise test finishes. [When mixing promise-based logic and async
steps](https://github.com/web-platform-tests/wpt/pull/17924), the next
test may begin to execute before the returned promise has settled. Use
[add_cleanup](#cleanup) to register any necessary cleanup actions such
as resetting global state that need to happen consistently before the
next test starts.

To test that a promise rejects with a specified exception see [promise
rejection](#promise-rejection).

### Single Page Tests ###

Sometimes, particularly when dealing with asynchronous behaviour,
having exactly one test per page is desirable, and the overhead of
wrapping everything in functions for isolation becomes
burdensome. For these cases `testharness.js` support "single page
tests".

In order for a test to be interpreted as a single page test, it should set the
`single_test` [setup option](#setup) to `true`.

```html
<!doctype html>
<title>Basic document.body test</title>
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<body>
  <script>
    setup({ single_test: true });
    assert_equals(document.body, document.getElementsByTagName("body")[0])
    done()
 </script>
```

The test title for single page tests is always taken from `document.title`.

## Making assertions ##

Functions for making assertions start `assert_`. The full list of
asserts available is documented in the [asserts](#assert-functions)
section. The general signature is:

```js
assert_something(actual, expected, description)
```

although not all assertions precisely match this pattern
e.g. [`assert_true`](#assert_true) only takes `actual` and
`description` as arguments.

The description parameter is used to present more useful error
messages when a test fails.

When assertions are violated, they throw an
[`AssertionError`](#AssertionError) exception. This interrupts test
execution, so subsequent statements are not evaluated. A given test
can only fail due to one such violation, so if you would like to
assert multiple behaviors independently, you should use multiple
tests.

**Note:** Unless the test is a [single page test](#single-page-tests),
assert functions must only be called in the context of a
[`Test`](#Test).

### Optional Features ###

If a test depends on a specification or specification feature that is
OPTIONAL (in the [RFC 2119
sense](https://tools.ietf.org/html/rfc2119)),
[`assert_implements_optional`](#assert_implements_optional) can be
used to indicate that failing the test does not mean violating a web
standard. For example:

```js
async_test((t) => {
  const video = document.createElement("video");
  assert_implements_optional(video.canPlayType("video/webm"));
  video.src = "multitrack.webm";
  // test something specific to multiple audio tracks in a WebM container
  t.done();
}, "WebM with multiple audio tracks");
```

A failing [`assert_implements_optional`](#assert_implements_optional)
call is reported as a status of `PRECONDITION_FAILED` for the
subtest. This unusual status code is a legacy leftover; see the [RFC
that introduced
`assert_implements_optional`](https://github.com/web-platform-tests/rfcs/pull/48).

[`assert_implements_optional`](#assert_implements_optional) can also
be used during [test setup](#setup). For example:

```js
setup(() => {
  assert_implements_optional("optionalfeature" in document.body,
                             "'optionalfeature' event supported");
});
async_test(() => { /* test #1 waiting for "optionalfeature" event */ });
async_test(() => { /* test #2 waiting for "optionalfeature" event */ });
```

A failing [`assert_implements_optional`](#assert_implements_optional)
during setup is reported as a status of `PRECONDITION_FAILED` for the
test, and the subtests will not run.

See also the `.optional` [file name convention](file-names.md), which may be
preferable if the entire test is optional.

## Testing Across Globals ##

### Consolidating tests from other documents ###

```eval_rst
.. js::autofunction fetch_tests_from_window
```

**Note:** By default any markup file referencing `testharness.js` will
be detected as a test. To avoid this, it must be put in a `support`
directory.

The current test suite will not report completion until all fetched
tests are complete, and errors in the child contexts will result in
failures for the suite in the current context.

Here's an example that uses `window.open`.

`support/child.html`:

```html
<!DOCTYPE html>
<html>
<title>Child context test(s)</title>
<head>
  <script src="/resources/testharness.js"></script>
</head>
<body>
  <div id="log"></div>
  <script>
    test(function(t) {
      assert_true(true, "true is true");
    }, "Simple test");
  </script>
</body>
</html>
```

`test.html`:

```html
<!DOCTYPE html>
<html>
<title>Primary test context</title>
<head>
  <script src="/resources/testharness.js"></script>
  <script src="/resources/testharnessreport.js"></script>
</head>
<body>
  <div id="log"></div>
  <script>
    var child_window = window.open("support/child.html");
    fetch_tests_from_window(child_window);
  </script>
</body>
</html>
```

### Web Workers ###

```eval_rst
.. js:autofunction fetch_tests_from_worker
```

The `testharness.js` script can be used from within [dedicated workers, shared
workers](https://html.spec.whatwg.org/multipage/workers.html) and [service
workers](https://w3c.github.io/ServiceWorker/).

Testing from a worker script is different from testing from an HTML document in
several ways:

* Workers have no reporting capability since they are running in the background.
  Hence they rely on `testharness.js` running in a companion client HTML document
  for reporting.

* Shared and service workers do not have a unique client document
  since there could be more than one document that communicates with
  these workers. So a client document needs to explicitly connect to a
  worker and fetch test results from it using
  [`fetch_tests_from_worker`](#fetch_tests_from_worker). This is true
  even for a dedicated worker. Once connected, the individual tests
  running in the worker (or those that have already run to completion)
  will be automatically reflected in the client document.

* The client document controls the timeout of the tests. All worker
  scripts act as if they were started with the
  [`explicit_timeout`](#setup) option.

* Dedicated and shared workers don't have an equivalent of an `onload`
  event.  Thus the test harness has no way to know when all tests have
  completed (see [Determining when all tests are
  complete](#determining-when-all-tests-are-complete)). So these
  worker tests behave as if they were started with the
  [`explicit_done`](#setup) option. Service workers depend on the
  [oninstall](https://w3c.github.io/ServiceWorker/#service-worker-global-scope-install-event)
  event and don't require an explicit [`done`](#done) call.

Here's an example that uses a dedicated worker.

`worker.js`:

```js
importScripts("/resources/testharness.js");

test(function(t) {
  assert_true(true, "true is true");
}, "Simple test");

// done() is needed because the testharness is running as if explicit_done
// was specified.
done();
```

`test.html`:

```html
<!DOCTYPE html>
<title>Simple test</title>
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<div id="log"></div>
<script>

fetch_tests_from_worker(new Worker("worker.js"));

</script>
```


`fetch_tests_from_worker` returns a promise that resolves once all the remote
tests have completed. This is useful if you're importing tests from multiple
workers and want to ensure they run in series:

```js
(async function() {
  await fetch_tests_from_worker(new Worker("worker-1.js"));
  await fetch_tests_from_worker(new Worker("worker-2.js"));
})();
```

## Cleanup ##

Occasionally tests may create state that will persist beyond the test
itself.  In order to ensure that tests are independent, such state
should be cleaned up once the test has a result. This can be achieved
by adding cleanup callbacks to the test. Such callbacks are registered
using the [`add_cleanup`](#Test.add_cleanup) method. All registered
callbacks will be run as soon as the test result is known. For
example:

```js
  test(function() {
    var element = document.createElement("div");
    element.setAttribute("id", "null");
    document.body.appendChild(element);
    this.add_cleanup(function() { document.body.removeChild(element) });
    assert_equals(document.getElementById(null), element);
  }, "Calling document.getElementById with a null argument.");
```

If the test was created using the [`promise_test`](#promise_test) API,
then cleanup functions may optionally return a Promise and delay the
completion of the test until all cleanup promises have settled.

All callbacks will be invoked synchronously; tests that require more
complex cleanup behavior should manage execution order explicitly. If
any of the eventual values are rejected, the test runner will report
an error.

### AbortSignal support ###

[`Test.get_signal`](#Test.get_signal) gives an AbortSignal that is aborted when
the test finishes. This can be useful when dealing with AbortSignal-supported
APIs.

```js
promise_test(t => {
  // Throws when the user agent does not support AbortSignal
  const signal = t.get_signal();
  const event = await new Promise(resolve => {
    document.body.addEventListener(resolve, { once: true, signal });
    document.body.click();
  });
  assert_equals(event.type, "click");
}, "");
```

## Timers in Tests ##

In general the use of timers (i.e. `setTimeout`) in tests is
discouraged because this is an observed source of instability on test
running in CI. In particular if a test should fail when
something doesn't happen, it is good practice to simply let the test
run to the full timeout rather than trying to guess an appropriate
shorter timeout to use.

In other cases it may be necessary to use a timeout (e.g., for a test
that only passes if some event is *not* fired). In this case it is
*not* permitted to use the standard `setTimeout` function. Instead use
either [`Test.step_wait()`](#Test.step_wait),
[`Test.step_wait_func()`](#Test.step_wait_func), or
[`Test.step_timeout()`](#Test.step_timeout). [`Test.step_wait()`](#Test.step_wait)
and [`Test.step_wait_func()`](#Test.step_wait_func) are preferred
when there's a specific condition that needs to be met for the test to
proceed. [`Test.step_timeout()`](#Test.step_timeout) is preferred in other cases.

Note that timeouts generally need to be a few seconds long in order to
produce stable results in all test environments.

For [single page tests](#single-page-tests),
[step_timeout](#step_timeout) is also available as a global function.

```eval_rst

.. js:autofunction:: <anonymous>~step_timeout
   :short-name:
```

## Harness Configuration ###

### Setup ###

<!-- sphinx-js doesn't support documenting types so we have to copy in
     the SettingsObject documentation by hand -->

```eval_rst
.. js:autofunction:: setup

.. js:autofunction:: promise_setup

:SettingsObject:

    :Properties:
        - **single_test** (*bool*) - Use the single-page-test mode. In this
          mode the Document represents a single :js:class:`Test`. Asserts may be
          used directly without requiring :js:func:`Test.step` or similar wrappers,
          and any exceptions set the status of the test rather than the status
          of the harness.

        - **allow_uncaught_exception** (*bool*) - don't treat an
          uncaught exception as an error; needed when e.g. testing the
          `window.onerror` handler.

        - **explicit_done** (*bool*) - Wait for a call to :js:func:`done`
          before declaring all tests complete (this is always true for
          single-page tests).

        - **hide_test_state** (*bool*) - hide the test state output while
          the test is running; This is helpful when the output of the test state
          may interfere the test results.

        - **explicit_timeout** (*bool*)  - disable file timeout; only
          stop waiting for results when the :js:func:`timeout` function is
          called. This should typically only be set for manual tests, or
          by a test runner that provides its own timeout mechanism.

        - **timeout_multiplier** (*Number*) - Multiplier to apply to
          timeouts. This should only be set by a test runner.

        - **output** (*bool*) - (default: `true`) Whether to output a table
          containing a summary of test results. This should typically
          only be set by a test runner, and is typically set to false
          for performance reasons when running in CI.

        - **output_document** (*Document*) output_document - The document to which
          results should be logged. By default this is the current
          document but could be an ancestor document in some cases e.g. a
          SVG test loaded in an HTML wrapper

        - **debug** (*bool*) - (default: `false`) Whether to output
          additional debugging information such as a list of
          asserts. This should typically only be set by a test runner.
```

### Output ###

If the file containing the tests is a HTML file, a table containing
the test results will be added to the document after all tests have
run. By default this will be added to a `div` element with `id=log` if
it exists, or a new `div` element appended to `document.body` if it
does not. This can be suppressed by setting the [`output`](#setup)
setting to `false`.

If [`output`](#setup) is `true`, the test will, by default, report
progress during execution. In some cases this progress report will
invalidate the test. In this case the test should set the
[`hide_test_state`](#setup) setting to `true`.


### Determining when all tests are complete ###

By default, tests running in a `WindowGlobalScope`, which are not
configured as a [single page test](#single-page-tests) the test
harness will assume there are no more results to come when:

 1. There are no `Test` objects that have been created but not completed
 2. The load event on the document has fired

For single page tests, or when the `explicit_done` property has been
set in the [setup](#setup), the [`done`](#done) function must be used.

```eval_rst

.. js:autofunction:: <anonymous>~done
   :short-name:
.. js:autofunction:: <anonymous>~timeout
   :short-name:
```

Dedicated and shared workers don't have an event that corresponds to
the `load` event in a document. Therefore these worker tests always
behave as if the `explicit_done` property is set to true (unless they
are defined using [the "multi-global"
pattern](testharness.html#multi-global-tests)). Service workers depend
on the
[install](https://w3c.github.io/ServiceWorker/#service-worker-global-scope-install-event)
event which is fired following the completion of [running the
worker](https://html.spec.whatwg.org/multipage/workers.html#run-a-worker).

## Reporting API ##

### Callbacks ###

The framework provides callbacks corresponding to 4 events:

 * `start` - triggered when the first Test is created
 * `test_state` - triggered when a test state changes
 * `result` - triggered when a test result is received
 * `complete` - triggered when all results are received

```eval_rst
.. js:autofunction:: add_start_callback
.. js:autofunction:: add_test_state_callback
.. js:autofunction:: add_result_callback
.. js:autofunction:: add_completion_callback
.. js:autoclass:: TestsStatus
   :members:
.. js:autoclass:: AssertRecord
   :members:
```

### External API ###

In order to collect the results of multiple pages containing tests, the test
harness will, when loaded in a nested browsing context, attempt to call
certain functions in each ancestor and opener browsing context:

 * start - `start_callback`
 * test\_state - `test_state_callback`
 * result - `result_callback`
 * complete - `completion_callback`

These are given the same arguments as the corresponding internal callbacks
described above.

The test harness will also send messages using cross-document
messaging to each ancestor and opener browsing context. Since it uses the
wildcard keyword (\*), cross-origin communication is enabled and script on
different origins can collect the results.

This API follows similar conventions as those described above only slightly
modified to accommodate message event API. Each message is sent by the harness
is passed a single vanilla object, available as the `data` property of the event
object. These objects are structured as follows:

 * start - `{ type: "start" }`
 * test\_state - `{ type: "test_state", test: Test }`
 * result - `{ type: "result", test: Test }`
 * complete - `{ type: "complete", tests: [Test, ...], status: TestsStatus }`


## Assert Functions ##

```eval_rst
.. js:autofunction:: assert_true
.. js:autofunction:: assert_false
.. js:autofunction:: assert_equals
.. js:autofunction:: assert_not_equals
.. js:autofunction:: assert_in_array
.. js:autofunction:: assert_array_equals
.. js:autofunction:: assert_approx_equals
.. js:autofunction:: assert_array_approx_equals
.. js:autofunction:: assert_less_than
.. js:autofunction:: assert_greater_than
.. js:autofunction:: assert_between_exclusive
.. js:autofunction:: assert_less_than_equal
.. js:autofunction:: assert_greater_than_equal
.. js:autofunction:: assert_between_inclusive
.. js:autofunction:: assert_regexp_match
.. js:autofunction:: assert_class_string
.. js:autofunction:: assert_own_property
.. js:autofunction:: assert_not_own_property
.. js:autofunction:: assert_inherits
.. js:autofunction:: assert_idl_attribute
.. js:autofunction:: assert_readonly
.. js:autofunction:: assert_throws_dom
.. js:autofunction:: assert_throws_js
.. js:autofunction:: assert_throws_exactly
.. js:autofunction:: assert_implements
.. js:autofunction:: assert_implements_optional
.. js:autofunction:: assert_unreached
.. js:autofunction:: assert_any

```

Assertions fail by throwing an `AssertionError`:

```eval_rst
.. js:autoclass:: AssertionError
```

### Promise Rejection ###

```eval_rst
.. js:autofunction:: promise_rejects_dom
.. js:autofunction:: promise_rejects_js
.. js:autofunction:: promise_rejects_exactly
```

`promise_rejects_dom`, `promise_rejects_js`, and `promise_rejects_exactly` can
be used to test Promises that need to reject.

Here's an example where the `bar()` function returns a Promise that rejects
with a TypeError:

```js
function bar() {
  return Promise.reject(new TypeError());
}

promise_test(function(t) {
  return promise_rejects_js(t, TypeError, bar());
}, "Another example");
```

## Test Objects ##

```eval_rst

.. js:autoclass:: Test
   :members:
```

## Helpers ##

### Waiting for events ###

```eval_rst
.. js:autoclass:: EventWatcher
   :members:
```

Here's an example of how to use `EventWatcher`:

```js
var t = async_test("Event order on animation start");

var animation = watchedNode.getAnimations()[0];
var eventWatcher = new EventWatcher(t, watchedNode, ['animationstart',
                                                     'animationiteration',
                                                     'animationend']);

eventWatcher.wait_for('animationstart').then(t.step_func(function() {
  assertExpectedStateAtStartOfAnimation();
  animation.currentTime = END_TIME; // skip to end
  // We expect two animationiteration events then an animationend event on
  // skipping to the end of the animation.
  return eventWatcher.wait_for(['animationiteration',
                                'animationiteration',
                                'animationend']);
})).then(t.step_func(function() {
  assertExpectedStateAtEndOfAnimation();
  t.done();
}));
```

### Loading test data from JSON files ###

```eval_rst
.. js:autofunction:: fetch_json
```

Loading test data from a JSON file would normally be accomplished by
something like this:

```js
promise_test(() => fetch('resources/my_data.json').then((res) => res.json()).then(runTests));
function runTests(myData) {
  /// ...
}
```

However, `fetch()` is not exposed inside ShadowRealm scopes, so if the
test is to be run inside a ShadowRealm, use `fetch_json()` instead:

```js
promise_test(() => fetch_json('resources/my_data.json').then(runTests));
```

### Utility Functions ###

```eval_rst
.. js:autofunction:: format_value
```

## Deprecated APIs ##

```eval_rst
.. js:autofunction:: generate_tests
.. js:autofunction:: on_event
```


