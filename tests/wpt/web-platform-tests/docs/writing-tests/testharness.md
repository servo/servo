# testharness.js Tests

```eval_rst
.. toctree::
   :maxdepth: 1

   idlharness
   testharness-api
   testdriver-extension-tutorial
   testdriver
```

testharness.js tests are the correct type of test to write in any
situation where you are not specifically interested in the rendering
of a page, and where human interaction isn't required; these tests are
written in JavaScript using a framework called `testharness.js`. It is
documented in two sections:

  * [testharness.js Documentation](testharness-api.md) — An introduction
    to the library and a detailed API reference.

  * [idlharness.js Documentation](idlharness.md) — A library for testing
     IDL interfaces using `testharness.js`.

See [server features](server-features.md) for advanced testing features that are commonly used
with testharness.js. See also the [general guidelines](general-guidelines.md) for all test types.

This page describes testharness.js exhaustively; [the tutorial on writing a
testharness.js test](testharness-tutorial) provides a concise guide to writing
a test--a good place to start for newcomers to the project.

## Variants

A test file can have multiple variants by including `meta` elements,
for example:

```html
<meta name="variant" content="">
<meta name="variant" content="?wss">
```

The test can then do different things based on the URL.

There are two utility scripts in that work well together with variants,
`/common/subset-tests.js` and `/common/subset-tests-by-key.js`, where
a test that would otherwise have too many tests to be useful can be
split up in ranges of subtests. For example:

```html
<!doctype html>
<title>Testing variants</title>
<meta name="variant" content="?1-1000">
<meta name="variant" content="?1001-2000">
<meta name="variant" content="?2001-last">
<script src="/resources/testharness.js">
<script src="/resources/testharnessreport.js">
<script src="/common/subset-tests.js">
<script>
 const tests = [
                 { fn: t => { ... }, name: "..." },
                 ... lots of tests ...
               ];
 for (const test of tests) {
   subsetTest(async_test, test.fn, test.name);
 }
</script>
```

With `subsetTestByKey`, the key is given as the first argument, and the
query string can include or exclude a key (will be matched as a regular
expression).

```html
<!doctype html>
<title>Testing variants by key</title>
<meta name="variant" content="?include=Foo">
<meta name="variant" content="?include=Bar">
<meta name="variant" content="?exclude=(Foo|Bar)">
<script src="/resources/testharness.js">
<script src="/resources/testharnessreport.js">
<script src="/common/subset-tests-by-key.js">
<script>
   subsetTestByKey("Foo", async_test, () => { ... }, "Testing foo");
   ...
</script>
```

## Auto-generated test boilerplate

While most JavaScript tests require a certain amount of HTML
boilerplate to include the test library, etc., tests which are
expressible purely in script (e.g. tests for workers) can have all the
needed HTML and script boilerplate auto-generated.

### Standalone window tests

Tests that only require a script file running in window scope can use
standalone window tests. In this case the test is a javascript file
with the extension `.window.js`. This is sourced from a generated
document which sources `testharness.js`, `testharnessreport.js` and
the test script. For a source script with the name
`example.window.js`, the corresponding test resource will be
`example.window.html`.

### Standalone workers tests

Tests that only require assertions in a dedicated worker scope can use
standalone workers tests. In this case, the test is a JavaScript file
with extension `.worker.js` that imports `testharness.js`. The test can
then use all the usual APIs, and can be run from the path to the
JavaScript file with the `.js` removed.

For example, one could write a test for the `FileReaderSync` API by
creating a `FileAPI/FileReaderSync.worker.js` as follows:

```js
importScripts("/resources/testharness.js");
test(function () {
    const blob = new Blob(["Hello"]);
    const fr = new FileReaderSync();
    assert_equals(fr.readAsText(blob), "Hello");
}, "FileReaderSync#readAsText.");
done();
```

This test could then be run from `FileAPI/FileReaderSync.worker.html`.

### Multi-global tests

Tests for features that exist in multiple global scopes can be written in a way
that they are automatically run in several scopes. In this case, the test is a
JavaScript file with extension `.any.js`, which can use all the usual APIs.

By default, the test runs in a window scope and a dedicated worker scope.

For example, one could write a test for the `Blob` constructor by
creating a `FileAPI/Blob-constructor.any.js` as follows:

```js
test(function () {
    const blob = new Blob();
    assert_equals(blob.size, 0);
    assert_equals(blob.type, "");
    assert_false(blob.isClosed);
}, "The Blob constructor.");
```

This test could then be run from `FileAPI/Blob-constructor.any.worker.html` as well
as `FileAPI/Blob-constructor.any.html`.

It is possible to customize the set of scopes with a metadata comment, such as

```
// META: global=sharedworker
//       ==> would run in the shared worker scope
// META: global=window,serviceworker
//       ==> would only run in the window and service worker scope
// META: global=dedicatedworker
//       ==> would run in the default dedicated worker scope
// META: global=worker
//       ==> would run in the dedicated, shared, and service worker scopes
```

For a test file <code><var>x</var>.any.js</code>, the available scope keywords
are:

* `window` (default): to be run at <code><var>x</var>.any.html</code>
* `dedicatedworker` (default): to be run at <code><var>x</var>.any.worker.html</code>
* `serviceworker`: to be run at <code><var>x</var>.any.serviceworker.html</code> (`.https` is implied)
* `sharedworker`: to be run at <code><var>x</var>.any.sharedworker.html</code>
* `jsshell`: to be run in a JavaScript shell, without access to the DOM
  (currently only supported in SpiderMonkey, and skipped in wptrunner)
* `worker`: shorthand for the dedicated, shared, and service worker scopes

To check if your test is run from a window or worker you can use the following two methods that will
be made available by the framework:

    self.GLOBAL.isWindow()
    self.GLOBAL.isWorker()

Although [the global `done` function must be explicitly invoked for most
dedicated worker tests and shared worker
tests](testharness-api.html#determining-when-all-tests-are-complete), it is
automatically invoked for tests defined using the "multi-global" pattern.

### Specifying a test title in auto-generated boilerplate tests

Use `// META: title=This is the title of the test` at the beginning of the resource.

### Including other JavaScript resources in auto-generated boilerplate tests

Use `// META: script=link/to/resource.js` at the beginning of the resource. For example,

```
// META: script=/common/utils.js
// META: script=resources/utils.js
```

can be used to include both the global and a local `utils.js` in a test.

### Specifying a timeout of long in auto-generated boilerplate tests

Use `// META: timeout=long` at the beginning of the resource.

### Specifying test [variants](#variants) in auto-generated boilerplate tests

Use `// META: variant=url-suffix` at the beginning of the resource. For example,

```
// META: variant=
// META: variant=?wss
```
