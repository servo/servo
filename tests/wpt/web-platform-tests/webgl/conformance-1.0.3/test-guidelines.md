Contributing WebGL conformance tests Guidelines
===============================================

Thank you for contributing to the WebGL conformance tests.
Please try to follow these guidelines when submitting a test.

*   If you're new to git [here's a terse set of instructions](http://www.khronos.org/webgl/wiki/Using_Github_To_Contribute "Using Github to Contribute").

*   All changes and/or new tests should go in the sdk/tests/conformance folder

The tests under conformance-suites are snapshots and are only to be updated by
the WebGL Working Group when "official" snapshots are taken.

*   Please use the Khronos Group License (MIT)

These lines appears at the top of every html and js file under sdk/tests/conformance

    <!--
    /*
    ** Copyright (c) 2014 The Khronos Group Inc.
    **
    ** Permission is hereby granted, free of charge, to any person obtaining a
    ** copy of this software and/or associated documentation files (the
    ** "Materials"), to deal in the Materials without restriction, including
    ** without limitation the rights to use, copy, modify, merge, publish,
    ** distribute, sublicense, and/or sell copies of the Materials, and to
    ** permit persons to whom the Materials are furnished to do so, subject to
    ** the following conditions:
    **
    ** The above copyright notice and this permission notice shall be included
    ** in all copies or substantial portions of the Materials.
    **
    ** THE MATERIALS ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
    ** EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
    ** MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
    ** IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
    ** CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
    ** TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
    ** MATERIALS OR THE USE OR OTHER DEALINGS IN THE MATERIALS.
    */
    -->

*   Please use code similar to the code in existing tests

    Ideally, copy an existing test and modify it for your new test. Try not to duplicate
    code that already exists where appropriate. In particular

    *   use the functions in WebGLTestUtils rather than duplicating functionality.

        In particular, as much as possible, keep the WebGL code in your test specific
        to the issue being tested and try to use the helper functions to handle
        common setup.

        Examples:

        *    to create a WebGL context call `WebGLTestUtils.create3DContext`. Passed nothing
             it will create an offscreen canvas. Passed a canvas element it will create
             a context on that element. Passed a string it will look up the canvas element
             with the matching id and create a context from that element.

        *    use `WebGLTestUtils.checkCanvas` or `WebGLTestUtils.checkCanvasRect` rather
             than checking rendering results by hand.

        *    use the various quad and draw functions

             *    `WebGLTestUtils.setupUnitQuad` and `WebGLTestUtils.clearAndDrawUnitQuad` for
                   simple drawing.

             *    `WebGLTestUtils.setupColorQuad`, `WebGLTestUtils.drawFloatColorQuad`, and
                  `WebGLTestUilts.drawUByteColorQuad` for drawing in a particular color.

             *    `WebGLTestUtils.setupIndexedQuad` and `WebGLTestUtils.clearAndDrawIndexedQuad`
                  if you need a higher subdivision of vertices and/or vertex colors.

             *    use `WebgLTestUtils.setupTexturedQuad` if you need a unit quad with texture coords.
                  By default the positions will be at location 0 and the texture coords at location 1.

        *    If you need a custom shader use `WebGLTestUtils.setupProgram`. Note that it takes
             the following arguments. `gl`, `shaders`, `opt_attribs`, `opt_locations` where:

             `gl` is the WebGL context.

             `shaders` are an array of either script element ids, shader source, or WebGLShader
             objects. The first element in the array is the vertex shader, the second the fragment
             shader.

             `opt_attribs` is an optional array of attribute names. If provided the named attributes
             will have their locations bound to their index in this array.

             `opt_locations` is an optional array of attribute locations. If provided each attribute
             name in `opt_attribs` is bound to the corresponding location in `opt_locations`.

        *    If you need to wait for a composite call `WebGLTestUtils.waitForComposite`.
             As compositing is a browser specific thing this provides a central place to
             update all tests that rely on compositing to function.

    *   Code/Tag Order

        Most tests run inline. They don't use window.onload or the load event. This works by placing
        the script tag inside the body, *after* the canvas and required divs.

            <canvas id="example"></canvas>
            <div id="description"></div>
            <div id="console"></div>
            <script>
            var wtu = WebGLDebugUtils;
            var gl = wtu.create3DContext("example");
            ...

    *   Ending Tests

        *   Tests that are short and run synchronously end with

                <script src="../../resources/js-test-post.js"></script>

        *   Tests that take a long time use setTimeout so as not to freeze the browser.

            Many browsers will terminate JavaScript that takes more than a few seconds to execute
            without returning control to the browser. The workaround is code like this

                var numTests = 10;
                var currenTest = 0;
                function runNextTest() {
                  if (currentTest == numTests) {
                    finishTest();  // Tells the harness you're done.
                    return;
                  }
                  // Run your test.
                  ...
                  ++currentTest;
                  setTimeout(runNextTest, 100);
                }
                runNextTest();

            Remember the tests need to run without timing out even and slow mobile devices.
            The harness resets the timeout timer every time a test reports success or failure
            so as long as some part of your test calls `testPassed` or `testFailed` or one of the
            many wrappers (`shouldXXX`, `glErrorShouldBe`, `WebGLTestUtils.checkCanvasXXX`, etc..)
            every so often the harness will not timeout your test.

        *   The test harness requires the global variable `successfullyParse` to be set to true.
            This usually appears at the end of a file.

                var successfullyParsed = true;

    *   Do not use browser specific code.

        *   Do not check the browser version. Use feature detection.

        *   If you do need feature detection consider putting it into WebGLTestUtils so that
            other tests can go through the same abstraction and the workaround is isolated
            to one place.

        *   Vendors may place test harness specific code in the testing infrastructure.

                resources/js-test-pre.js
                conformance/more/unit.js

    *   Indent with spaces not tabs. (not everyone uses your tab settings).

    *   All HTML files must have a `<!DOCTYPE html>`

    *   All HTML files must have a `<meta charset="utf-8">`

    *   All JavaScript must start with "use strict";

*   If adding a new test edit the appropriate 00_test_list.txt file

    Each folder has a 00_test_list.txt file that lists the test in that folder.
    Each new test should be prefixed with the option `--min-version <version>` where
    version is 1 more than the newest official version. At the time of this writing
    all new tests should be prefixed with `--min-version 1.0.2`


