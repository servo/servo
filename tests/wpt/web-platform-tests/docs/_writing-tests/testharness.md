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
boilerplate to include the test library, etc., tests for Web Workers
can be written as JavaScript files which then have all the needed HTML
and setup script auto-generated.

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
JavaScript file with the `.js` replaced by `.worker.js` or `.html`.

For example, one could write a test for the `Blob` constructor by
creating a `FileAPI/Blob-constructor.any.js` as follows:

    test(function () {
      var blob = new Blob();
      assert_equals(blob.size, 0);
      assert_equals(blob.type, "");
      assert_false(blob.isClosed);
    }, "The Blob constructor.");

This test could then be run from `FileAPI/Blob-constructor.any.worker.js` as well
as `FileAPI/Blob-constructor.any.html`.


[general guidelines]: {{ site.baseurl }}{% link _writing-tests/general-guidelines.md %}
[testharness-api]: {{ site.baseurl }}{% link _writing-tests/testharness-api.html %}
[idlharness]: {{ site.baseurl }}{% link _writing-tests/idlharness.html %}
