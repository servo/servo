---
layout: page
title: testharness.js API
order: 5
---

## Introduction ##

testharness.js provides a framework for writing testcases. It is intended to
provide a convenient API for making common assertions, and to work both
for testing synchronous and asynchronous DOM features in a way that
promotes clear, robust, tests.

## Basic Usage ##

The test harness script can be used from HTML or SVG documents and web worker
scripts.

From an HTML or SVG document, start by importing both `testharness.js` and
`testharnessreport.js` scripts into the document:

```html
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
```

Refer to the [Web Workers](#web-workers) section for details and an example on
testing within a web worker.

Within each file one may define one or more tests. Each test is atomic in the
sense that a single test has a single result (`PASS`/`FAIL`/`TIMEOUT`/`NOTRUN`).
Within each test one may have a number of asserts. The test fails at the first
failing assert, and the remainder of the test is (typically) not run.

If the file containing the tests is a HTML file, a table containing the test
results will be added to the document after all tests have run. By default this
will be added to a `div` element with `id=log` if it exists, or a new `div`
element appended to `document.body` if it does not.

NOTE: By default tests must be created before the load event fires. For ways
to create tests after the load event, see "Determining when all tests
are complete", below.

## Synchronous Tests ##

To create a synchronous test use the `test()` function:

```js
test(test_function, name, properties)
```

`test_function` is a function that contains the code to test. For example a
trivial test for the DOM
[`hasFeature()`](https://dom.spec.whatwg.org/#dom-domimplementation-hasfeature)
method (which is defined to always return true) would be:

```js
test(function() {
  assert_true(document.implementation.hasFeature());
}, "hasFeature() with no arguments")
```

The function passed in is run in the `test()` call.

`properties` is a javascript object for passing extra options to the
test. Currently it is only used to provide test-specific
metadata, as described in the [metadata](#metadata) section below.

## Asynchronous Tests ##

Testing asynchronous features is somewhat more complex since the result of
a test may depend on one or more events or other callbacks. The API provided
for testing these features is intended to be rather low-level but hopefully
applicable to many situations.

To create a test, one starts by getting a `Test` object using `async_test`:

```js
async_test(name, properties)
```

e.g.

```js
var t = async_test("DOMContentLoaded")
```

Assertions can be added to the test by calling the step method of the test
object with a function containing the test assertions:

```js
document.addEventListener("DOMContentLoaded", function() {
  t.step(function() {
    assert_true(e.bubbles, "bubbles should be true");
  });
});
```

When all the steps are complete, the `done()` method must be called:

```js
t.done();
```

As a convenience, `async_test` can also takes a function as first argument.
This function is called with the test object as both its `this` object and
first argument. The above example can be rewritten as:

```js
async_test(function(t) {
  document.addEventListener("DOMContentLoaded", function() {
    t.step(function() {
      assert_true(e.bubbles, "bubbles should be true");
    });
    t.done();
  });
}, "DOMContentLoaded");
```

which avoids cluttering the global scope with references to async
tests instances.

The properties argument is identical to that for `test()`.

In many cases it is convenient to run a step in response to an event or a
callback. A convenient method of doing this is through the `step_func` method
which returns a function that, when called runs a test step. For example:

```js
document.addEventListener("DOMContentLoaded", t.step_func(function() {
  assert_true(e.bubbles, "bubbles should be true");
  t.done();
}));
```

As a further convenience, the `step_func` that calls `done()` can instead
use `step_func_done`, as follows:

```js
document.addEventListener("DOMContentLoaded", t.step_func_done(function() {
  assert_true(e.bubbles, "bubbles should be true");
}));
```

For asynchronous callbacks that should never execute, `unreached_func` can
be used. For example:

```js
document.documentElement.addEventListener("DOMContentLoaded",
  t.unreached_func("DOMContentLoaded should not be fired on the document element"));
```

Keep in mind that other tests could start executing before an Asynchronous
Test is finished.

## Promise Tests ##

`promise_test` can be used to test APIs that are based on Promises:

```js
promise_test(test_function, name, properties)
```

`test_function` is a function that receives a test as an argument and returns a
promise. The test completes when the returned promise resolves. The test fails
if the returned promise rejects.

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

Note that in the promise chain constructed in `test_function` assertions don't
need to be wrapped in `step` or `step_func` calls.

Unlike Asynchronous Tests, Promise Tests don't start running until after the
previous Promise Test finishes.

`promise_rejects` can be used to test Promises that need to reject:

```js
promise_rejects(test_object, code, promise, description)
```

The `code` argument is equivalent to the same argument to the `assert_throws`
function.

Here's an example where the `bar()` function returns a Promise that rejects
with a TypeError:

```js
function bar() {
  return Promise.reject(new TypeError());
}

promise_test(function(t) {
  return promise_rejects(t, new TypeError(), bar());
}, "Another example");
```

`EventWatcher` is a constructor function that allows DOM events to be handled
using Promises, which can make it a lot easier to test a very specific series
of events, including ensuring that unexpected events are not fired at any point.

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

`wait_for` either takes the name of a single event and returns a Promise that
will resolve after that event is fired at the watched node, or else it takes an
array of the names of a series of events and returns a Promise that will
resolve after that specific series of events has been fired at the watched node.

`EventWatcher` will assert if an event occurs while there is no `wait_for`()
created Promise waiting to be fulfilled, or if the event is of a different type
to the type currently expected. This ensures that only the events that are
expected occur, in the correct order, and with the correct timing.

## Single Page Tests ##

Sometimes, particularly when dealing with asynchronous behaviour,
having exactly one test per page is desirable, and the overhead of
wrapping everything in functions for isolation becomes
burdensome. For these cases `testharness.js` support "single page
tests".

In order for a test to be interpreted as a single page test, then
it must simply not call `test()` or `async_test()` anywhere on the page, and
must call the `done()` function to indicate that the test is complete. All
the `assert_*` functions are available as normal, but are called without
the normal step function wrapper. For example:

```html
<!doctype html>
<title>Basic document.body test</title>
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<body>
  <script>
    assert_equals(document.body, document.getElementsByTagName("body")[0])
    done()
 </script>
```

The test title for single page tests is always taken from `document.title`.

## Making assertions ##

Functions for making assertions start `assert_`. The full list of
asserts available is documented in the [asserts](#list-of-assertions) section
below. The general signature is:

```js
assert_something(actual, expected, description)
```

although not all assertions precisely match this pattern e.g. `assert_true`
only takes `actual` and `description` as arguments.

The description parameter is used to present more useful error messages when
a test fails.

NOTE: All asserts must be located in a `test()` or a step of an
`async_test()`, unless the test is a single page test. Asserts outside
these places won't be detected correctly by the harness and may cause
unexpected exceptions that will lead to an error in the harness.

## Cleanup ##

Occasionally tests may create state that will persist beyond the test itself.
In order to ensure that tests are independent, such state should be cleaned
up once the test has a result. This can be achieved by adding cleanup
callbacks to the test. Such callbacks are registered using the `add_cleanup`
function on the test object. All registered callbacks will be run as soon as
the test result is known. For example:

```js
  test(function() {
    var element = document.createElement("div");
    element.setAttribute("id", "null");
    document.body.appendChild(element);
    this.add_cleanup(function() { document.body.removeChild(element) });
    assert_equals(document.getElementById(null), element);
  }, "Calling document.getElementById with a null argument.");
```

## Timeouts in Tests ##

In general the use of timeouts in tests is discouraged because this is
an observed source of instability in real tests when run on CI
infrastructure. In particular if a test should fail when something
doesn't happen, it is good practice to simply let the test run to the
full timeout rather than trying to guess an appropriate shorter
timeout to use.

In other cases it may be necessary to use a timeout (e.g., for a test
that only passes if some event is *not* fired). In this case it is
*not* permitted to use the standard `setTimeout` function. Instead one
must use the `step_timeout` function:

```js
async_test(function(t) {
  var gotEvent = false;
  document.addEventListener("DOMContentLoaded", t.step_func(function() {
    assert_false(gotEvent, "Unexpected DOMContentLoaded event");
    gotEvent = true;
    t.step_timeout(function() { t.done(); }, 2000);
  });
}, "Only one DOMContentLoaded");
```

The difference between `setTimeout` and `step_timeout` is that the
latter takes account of the timeout multiplier when computing the
delay; e.g., in the above case a timeout multiplier of 2 would cause a
pause of 4000ms before calling the callback. This makes it less likely
to produce unstable results in slow configurations.

Note that timeouts generally need to be a few seconds long in order to
produce stable results in all test environments.

For single-page tests, `step_timeout` is also available as a global
function.

## Harness Timeout ##

The overall harness admits two timeout values `"normal"` (the
default) and `"long"`, used for tests which have an unusually long
runtime. After the timeout is reached, the harness will stop
waiting for further async tests to complete. By default the
timeouts are set to 10s and 60s, respectively, but may be changed
when the test is run on hardware with different performance
characteristics to a common desktop computer.  In order to opt-in
to the longer test timeout, the test must specify a meta element:

```html
<meta name="timeout" content="long">
```

Occasionally tests may have a race between the harness timing out and
a particular test failing; typically when the test waits for some event
that never occurs. In this case it is possible to use `test.force_timeout()`
in place of `assert_unreached()`, to immediately fail the test but with a
status of `TIMEOUT`. This should only be used as a last resort when it is
not possible to make the test reliable in some other way.

## Setup ##

Sometimes tests require non-trivial setup that may fail. For this purpose
there is a `setup()` function, that may be called with one or two arguments.
The two argument version is:

```js
setup(func, properties)
```

The one argument versions may omit either argument.
func is a function to be run synchronously. `setup()` becomes a no-op once
any tests have returned results. Properties are global properties of the test
harness. Currently recognised properties are:

`explicit_done` - Wait for an explicit call to done() before declaring all
tests complete (see below; implicitly true for single page tests)

`output_document` - The document to which results should be logged. By default
this is the current document but could be an ancestor document in some cases
e.g. a SVG test loaded in an HTML wrapper

`explicit_timeout` - disable file timeout; only stop waiting for results
when the `timeout()` function is called (typically for use when integrating
with some existing test framework that has its own timeout mechanism).

`allow_uncaught_exception` - don't treat an uncaught exception as an error;
needed when e.g. testing the `window.onerror` handler.

`timeout_multiplier` - Multiplier to apply to per-test timeouts.

## Determining when all tests are complete ##

By default the test harness will assume there are no more results to come
when:

 1. There are no `Test` objects that have been created but not completed
 2. The load event on the document has fired

This behaviour can be overridden by setting the `explicit_done` property to
true in a call to `setup()`. If `explicit_done` is true, the test harness will
not assume it is done until the global `done()` function is called. Once `done()`
is called, the two conditions above apply like normal.

Dedicated and shared workers don't have an event that corresponds to the `load`
event in a document. Therefore these worker tests always behave as if the
`explicit_done` property is set to true. Service workers depend on the
[install](https://slightlyoff.github.io/ServiceWorker/spec/service_worker/index.html#service-worker-global-scope-install-event)
event which is fired following the completion of [running the
worker](https://html.spec.whatwg.org/multipage/workers.html#run-a-worker).

## **DEPRECATED** Generating tests ##

There are scenarios in which is is desirable to create a large number of
(synchronous) tests that are internally similar but vary in the parameters
used. To make this easier, the `generate_tests` function allows a single
function to be called with each set of parameters in a list:

```js
generate_tests(test_function, parameter_lists, properties)
```

For example:

```js
generate_tests(assert_equals, [
    ["Sum one and one", 1+1, 2],
    ["Sum one and zero", 1+0, 1]
    ])
```

Is equivalent to:

```js
var a_got = 1+1;
var a_exp = 2;
var a_nam = "Sum one and one";
var b_got = 1+0;
var b_exp = 1;
var b_nam = "Sum one and zero";
test(function() {assert_equals(a_got, a_exp)}, a_nam);
test(function() {assert_equals(b_got, b_exp)}, b_nam);
```

Note that the first item in each parameter list corresponds to the name of
the test.

The properties argument is identical to that for `test()`. This may be a
single object (used for all generated tests) or an array.

This is deprecated because it runs all the tests outside of the test functions
and as a result any test throwing an exception will result in no tests being
run. In almost all cases, you should simply call test within the loop you would
use to generate the parameter list array.

## Callback API ##

The framework provides callbacks corresponding to 4 events:

 * `start` - triggered when the first Test is created
 * `test_state` - triggered when a test state changes
 * `result` - triggered when a test result is received
 * `complete` - triggered when all results are received

The page defining the tests may add callbacks for these events by calling
the following methods:

  `add_start_callback(callback)` - callback called with no arguments

  `add_test_state_callback(callback)` - callback called with a test argument

  `add_result_callback(callback)` - callback called with a test argument

  `add_completion_callback(callback)` - callback called with an array of tests
                                        and a status object

Tests have the following properties:

  * `status` - A status code. This can be compared to the `PASS`, `FAIL`,
               `TIMEOUT` and `NOTRUN` properties on the test object

  * `message` - A message indicating the reason for failure. In the future this
                will always be a string

 The status object gives the overall status of the harness. It has the
 following properties:

 * `status` - Can be compared to the `OK`, `ERROR` and `TIMEOUT` properties

 * `message` - An error message set when the status is `ERROR`

## External API ##

In order to collect the results of multiple pages containing tests, the test
harness will, when loaded in a nested browsing context, attempt to call
certain functions in each ancestor and opener browsing context:

 * start - `start_callback`
 * test\_state - `test_state_callback`
 * result - `result_callback`
 * complete - `completion_callback`

These are given the same arguments as the corresponding internal callbacks
described above.

## External API through cross-document messaging ##

Where supported, the test harness will also send messages using cross-document
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

## Consolidating tests from other documents ##

`fetch_tests_from_window` will aggregate tests from separate windows or iframes
into the current document as if they were all part of the same test suite. The
document of the second window (or iframe) should include `testharness.js`, but
not `testharnessreport.js`, and use `test`, `async_test`, and `promise_test` in
the usual manner.

The current test suite will not report completion until
all fetched tests are complete, and errors in the child contexts will result in
failures for the suite in the current context.

Here's an example that uses `window.open`.

`child.html`:

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
    var child_window = window.open("child.html");
    fetch_tests_from_window(child_window);
  </script>
</body>
</html>
```

The argument to `fetch_tests_from_window` is any [`Window`](https://html.spec.whatwg.org/multipage/browsers.html#the-window-object)
capable of accessing the browsing context as either an ancestor or opener.

## Web Workers ##

The `testharness.js` script can be used from within [dedicated workers, shared
workers](https://html.spec.whatwg.org/multipage/workers.html) and [service
workers](https://slightlyoff.github.io/ServiceWorker/spec/service_worker/).

Testing from a worker script is different from testing from an HTML document in
several ways:

* Workers have no reporting capability since they are running in the background.
  Hence they rely on `testharness.js` running in a companion client HTML document
  for reporting.

* Shared and service workers do not have a unique client document since there
  could be more than one document that communicates with these workers. So a
  client document needs to explicitly connect to a worker and fetch test results
  from it using `fetch_tests_from_worker`. This is true even for a dedicated
  worker. Once connected, the individual tests running in the worker (or those
  that have already run to completion) will be automatically reflected in the
  client document.

* The client document controls the timeout of the tests. All worker scripts act
  as if they were started with the `explicit_timeout` option (see the [Harness
  timeout](#harness-timeout) section).

* Dedicated and shared workers don't have an equivalent of an `onload` event.
  Thus the test harness has no way to know when all tests have completed (see
  [Determining when all tests are
  complete](#determining-when-all-tests-are-complete)). So these worker tests
  behave as if they were started with the `explicit_done` option. Service
  workers depend on the
  [oninstall](https://slightlyoff.github.io/ServiceWorker/spec/service_worker/index.html#service-worker-global-scope-install-event)
  event and don't require an explicit `done` call.

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

The argument to the `fetch_tests_from_worker` function can be a
[`Worker`](https://html.spec.whatwg.org/multipage/workers.html#dedicated-workers-and-the-worker-interface),
a [`SharedWorker`](https://html.spec.whatwg.org/multipage/workers.html#shared-workers-and-the-sharedworker-interface)
or a [`ServiceWorker`](https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-obj).
Once called, the containing document fetches all the tests from the worker and
behaves as if those tests were running in the containing document itself.

`fetch_tests_from_worker` returns a promise that resolves once all the remote
tests have completed. This is useful if you're importing tests from multiple
workers and want to ensure they run in series:

```js
(async function() {
  await fetch_tests_from_worker(new Worker("worker-1.js"));
  await fetch_tests_from_worker(new Worker("worker-2.js"));
})();
```

## List of Assertions ##

### `assert_true(actual, description)`
asserts that `actual` is strictly true

### `assert_false(actual, description)`
asserts that `actual` is strictly false

### `assert_equals(actual, expected, description)`
asserts that `actual` is the same value as `expected`.
Relies on `===`, distinguishes between `-0` and `+0`, and has a specific check for `NaN`.

### `assert_not_equals(actual, expected, description)`
asserts that `actual` is a different value to `expected`.
This means that `expected` is a misnomer.
Relies on `===`, distinguishes between `-0` and `+0`, and has a specific check for `NaN`.

### `assert_in_array(actual, expected, description)`
asserts that `expected` is an Array, and `actual` is equal to one of the
members i.e. `expected.indexOf(actual) != -1`

### `assert_array_equals(actual, expected, description)`
asserts that `actual` and `expected` have the same
length and the value of each indexed property in `actual` is the strictly equal
to the corresponding property value in `expected`

### `assert_array_approx_equals(actual, expected, epsilon, description)`
asserts that `actual` and `expected` have the same
length and each indexed property in `actual` is a number
within ±`epsilon` of the corresponding property in `expected`

### `assert_approx_equals(actual, expected, epsilon, description)`
asserts that `actual` is a number within ±`epsilon` of `expected`

### `assert_less_than(actual, expected, description)`
asserts that `actual` is a number less than `expected`

### `assert_greater_than(actual, expected, description)`
asserts that `actual` is a number greater than `expected`

### `assert_between_exclusive(actual, lower, upper, description`
asserts that `actual` is a number between `lower` and `upper` but not
equal to either of them

### `assert_less_than_equal(actual, expected, description)`
asserts that `actual` is a number less than or equal to `expected`

### `assert_greater_than_equal(actual, expected, description)`
asserts that `actual` is a number greater than or equal to `expected`

### `assert_between_inclusive(actual, lower, upper, description`
asserts that `actual` is a number between `lower` and `upper` or
equal to either of them

### `assert_regexp_match(actual, expected, description)`
asserts that `actual` matches the regexp `expected`

### `assert_class_string(object, class_name, description)`
asserts that the class string of `object` as returned in
`Object.prototype.toString` is equal to `class_name`.

### `assert_own_property(object, property_name, description)`
assert that object has own property `property_name`

### `assert_inherits(object, property_name, description)`
assert that object does not have an own property named
`property_name` but that `property_name` is present in the prototype
chain for object

### `assert_idl_attribute(object, attribute_name, description)`
assert that an object that is an instance of some interface has the
attribute attribute_name following the conditions specified by WebIDL

### `assert_readonly(object, property_name, description)`
assert that property `property_name` on object is readonly

### `assert_throws(code, func, description)`
`code` - the expected exception. This can take several forms:

  * string - the thrown exception must be a DOMException with the given
             name, e.g., "TimeoutError" (for compatibility with existing
             tests, a constant is also supported, e.g., "TIMEOUT_ERR")
  * object - the thrown exception must have a property called "name" that
             matches code.name

`func` - a function that should throw

### `assert_unreached(description)`
asserts if called. Used to ensure that some codepath is *not* taken e.g.
an event does not fire.

### `assert_any(assert_func, actual, expected_array, extra_arg_1, ... extra_arg_N)`
asserts that one `assert_func(actual, expected_array_N, extra_arg1, ..., extra_arg_N)`
  is true for some `expected_array_N` in `expected_array`. This only works for `assert_func`
  with signature `assert_func(actual, expected, args_1, ..., args_N)`. Note that tests
  with multiple allowed pass conditions are bad practice unless the spec specifically
  allows multiple behaviours. Test authors should not use this method simply to hide
  UA bugs.

### `assert_exists(object, property_name, description)`
**deprecated**
asserts that object has an own property `property_name`

### `assert_not_exists(object, property_name, description)`
**deprecated**
assert that object does not have own property `property_name`

## Metadata ##

It is possible to add optional metadata to tests; this can be done in
one of two ways; either by adding `<meta>` elements to the head of the
document containing the tests, or by adding the metadata to individual
`[async_]test` calls, as properties.
