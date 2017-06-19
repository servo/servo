---
layout: page
title: testharness.js Tests
order: 4
---

testharness.js tests are the correct type of test to write in any
situation where you are not specifically interested in the rendering
of a page, and where human interaction isn't required; these tests are
written in JavaScript using a framework called `testharness.js`. It is
documented in two sections:

  * [testharness.js Documentation][testharness-api] — An introduction
    to the library and a detailed API reference.

  * [idlharness.js Documentation][idlharness] — A library for testing
     IDL interfaces using `testharness.js`.

As always, we recommend reading over the [general guidelines][] for
all test types.

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

    importScripts("/resources/testharness.js");
    test(function () {
      var blob = new Blob(["Hello"]);
      var fr = new FileReaderSync();
      assert_equals(fr.readAsText(blob), "Hello");
    }, "FileReaderSync#readAsText.");
    done();

This test could then be run from `FileAPI/FileReaderSync.worker.html`.

### Multi-global tests

Tests for features that exist in multiple global scopes can be written
in a way that they are automatically run in a window scope and a
worker scope.

In this case, the test is a JavaScript file with extension `.any.js`.
The test can then use all the usual APIs, and can be run from the path to the
JavaScript file with the `.js` replaced by `.worker.html` or `.html`.

For example, one could write a test for the `Blob` constructor by
creating a `FileAPI/Blob-constructor.any.js` as follows:

    test(function () {
      var blob = new Blob();
      assert_equals(blob.size, 0);
      assert_equals(blob.type, "");
      assert_false(blob.isClosed);
    }, "The Blob constructor.");

This test could then be run from `FileAPI/Blob-constructor.any.worker.html` as well
as `FileAPI/Blob-constructor.any.html`.

To check if your test is run from a window or worker you can use the following two methods that will
be made available by the framework:

    self.GLOBAL.isWindow()
    self.GLOBAL.isWorker()

### Including other JavaScript resources in auto-generated boilerplate tests

Use `// META: script=link/to/resource.js` at the beginning of the resource. For example,

    // META: script=/common/utils.js
    // META: script=resources/utils.js

can be used to include both the global and a local `utils.js` in a test.

### Specifying a timeout of long in auto-generated boilerplate tests

Use `// META: timeout=long` at the beginning of the resource.


[general guidelines]: {{ site.baseurl }}{% link _writing-tests/general-guidelines.md %}
[testharness-api]: {{ site.baseurl }}{% link _writing-tests/testharness-api.md %}
[idlharness]: {{ site.baseurl }}{% link _writing-tests/idlharness.md %}
