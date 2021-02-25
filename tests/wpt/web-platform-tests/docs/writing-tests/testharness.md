# JavaScript Tests (testharness.js)

JavaScript tests are the correct type of test to write in any
situation where you are not specifically interested in the rendering
of a page, and where human interaction isn't required; these tests are
written in JavaScript using a framework called `testharness.js`.

A high-level overview is provided below and more information can be found here:

  * [testharness.js Documentation](testharness-api.md) — An introduction
    to the library and a detailed API reference. [The tutorial on writing a
    testharness.js test](testharness-tutorial) provides a concise guide to writing
    a test — a good place to start for newcomers to the project.

  * [testdriver.js Automation](testdriver.md) — Automating end user actions, such as moving or
    clicking a mouse. See also the
    [testdriver.js extension tutorial](testdriver-extension-tutorial.md) for adding new commands.

  * [idlharness.js Documentation](idlharness.md) — A library for testing
     IDL interfaces using `testharness.js`.

See [server features](server-features.md) for advanced testing features that are commonly used
with JavaScript tests. See also the [general guidelines](general-guidelines.md) for all test types.

## Window tests

### Without HTML boilerplate (`.window.js`)

Create a JavaScript file whose filename ends in `.window.js` to have the necessary HTML boilerplate
generated for you at `.window.html`. I.e., for `test.window.js` the server will ensure
`test.window.html` is available.

In this JavaScript file you can place one or more tests, as follows:
```js
test(() => {
  // Place assertions and logic here
  assert_equals(document.characterSet, "UTF-8");
}, "Ensure HTML boilerplate uses UTF-8"); // This is the title of the test
```

If you only need to test a [single thing](testharness-api.md#single-page-tests), you could also use:
```js
// META: title=Ensure HTML boilerplate uses UTF-8
setup({ single_test: true });
assert_equals(document.characterSet, "UTF-8");
done();
```

See [asynchronous (`async_test()`)](testharness-api.md#asynchronous-tests) and
[promise tests (`promise_test()`)](testharness-api.md#promise-tests) for more involved setups.

### With HTML boilerplate

You need to be a bit more explicit and include the `testharness.js` framework directly as well as an
additional file used by implementations:

```html
<!doctype html>
<meta charset=utf-8>
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<body>
  <script>
    test(() => {
      assert_equals(document.characterSet, "UTF-8");
    }, "Ensure UTF-8 declaration is observed");
  </script>
```

Here too you could avoid the wrapper `test()` function:

```html
<!doctype html>
<meta charset=utf-8>
<title>Ensure UTF-8 declaration is observed</title>
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<body>
  <script>
    setup({ single_test: true });
    assert_equals(document.characterSet, "UTF-8");
    done();
  </script>
```

In this case the test title is taken from the `title` element.

## Dedicated worker tests (`.worker.js`)

Create a JavaScript file that imports `testharness.js` and whose filename ends in `.worker.js` to
have the necessary HTML boilerplate generated for you at `.worker.html`.

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

(Removing the need for `importScripts()` and `done()` is tracked in
[issue #11529](https://github.com/web-platform-tests/wpt/issues/11529).)

## Tests for other or multiple globals (`.any.js`)

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
// META: global=dedicatedworker-module
//       ==> would run in the dedicated worker scope as a module
// META: global=worker
//       ==> would run in the dedicated, shared, and service worker scopes
```

For a test file <code><var>x</var>.any.js</code>, the available scope keywords
are:

* `window` (default): to be run at <code><var>x</var>.any.html</code>
* `dedicatedworker` (default): to be run at <code><var>x</var>.any.worker.html</code>
* `dedicatedworker-module` to be run at <code><var>x</var>.any.worker-module.html</code>
* `serviceworker`: to be run at <code><var>x</var>.any.serviceworker.html</code> (`.https` is
  implied)
* `serviceworker-module`: to be run at <code><var>x</var>.any.serviceworker-module.html</code>
  (`.https` is implied)
* `sharedworker`: to be run at <code><var>x</var>.any.sharedworker.html</code>
* `sharedworker-module`: to be run at <code><var>x</var>.any.sharedworker-module.html</code>
* `jsshell`: to be run in a JavaScript shell, without access to the DOM
  (currently only supported in SpiderMonkey, and skipped in wptrunner)
* `worker`: shorthand for the dedicated, shared, and service worker scopes

To check if your test is run from a window or worker you can use the following two methods that will
be made available by the framework:

    self.GLOBAL.isWindow()
    self.GLOBAL.isWorker()

Although [the global `done()` function must be explicitly invoked for most
dedicated worker tests and shared worker
tests](testharness-api.html#determining-when-all-tests-are-complete), it is
automatically invoked for tests defined using the "multi-global" pattern.

## Other features of `.window.js`, `.worker.js` and `.any.js`

### Specifying a test title

Use `// META: title=This is the title of the test` at the beginning of the resource.

### Including other JavaScript files

Use `// META: script=link/to/resource.js` at the beginning of the resource. For example,

```
// META: script=/common/utils.js
// META: script=resources/utils.js
```

can be used to include both the global and a local `utils.js` in a test.

In window environments, the script will be included using a classic `<script>` tag. In classic
worker environments, the script will be imported using `importScripts()`. In module worker
environments, the script will be imported using a static `import`.

### Specifying a timeout of long

Use `// META: timeout=long` at the beginning of the resource.

### Specifying test [variants](#variants)

Use `// META: variant=url-suffix` at the beginning of the resource. For example,

```
// META: variant=
// META: variant=?wss
```

## Variants

A test file can have multiple variants by including `meta` elements,
for example:

```html
<meta name="variant" content="">
<meta name="variant" content="?wss">
```

Test runners will execute the test for each variant specified, appending the corresponding content
attribute value to the URL of the test as they do so.

`/common/subset-tests.js` and `/common/subset-tests-by-key.js` are two utility scripts that work
well together with variants, allowing a test to be split up into subtests in cases when there are
otherwise too many tests to complete inside the timeout. For example:

```html
<!doctype html>
<title>Testing variants</title>
<meta name="variant" content="?1-1000">
<meta name="variant" content="?1001-2000">
<meta name="variant" content="?2001-last">
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
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
query string can include or exclude a key (which will be matched as a regular
expression).

```html
<!doctype html>
<title>Testing variants by key</title>
<meta name="variant" content="?include=Foo">
<meta name="variant" content="?include=Bar">
<meta name="variant" content="?exclude=(Foo|Bar)">
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<script src="/common/subset-tests-by-key.js"></script>
<script>
   subsetTestByKey("Foo", async_test, () => { ... }, "Testing foo");
   ...
</script>
```

## Table of Contents

```eval_rst
.. toctree::
   :maxdepth: 1

   testharness-api
   testdriver
   testdriver-extension-tutorial
   idlharness
```
