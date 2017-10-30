Tests for the WebGL canvas context
==================================

These tests are intended to serve the following purposes:

  * Assert spec conformance
  * Check the safety of the GL binding (bounds checking, same origin policy)
  * Provide performance numbers for developers


Running the tests
-----------------

  1. <a href="http://learningwebgl.com/blog/?p=11">Install a browser with WebGL support</a>
  2. Run <code>ruby gen_tests.rb</code> if you have modified the tests.
  3. Run <code>ruby test_server.rb</code> if you want to get test run output to test_server's stdout (especially useful for finding out which test crashed your browser.)
  4. Open all_tests.html in your browser.


Want to contribute?
-------------------

  1. Fork this repo
  2. Run <code>gen_tests.rb</code>
  3. Look into templates/ to see which functions lack tests (also see <a href="../raw/master/methods.txt">methods.txt</a> and <a href="http://dxr.mozilla.org/mozilla-central/source/dom/interfaces/canvas/nsICanvasRenderingContextWebGL.idl">nsICanvasRenderingContextWebGL.idl</a>):
    1. copy methodName.html to functions/methodName.html and write tests that test the results of valid inputs.
    2. copy methodNameBadArgs.html to functions/methodNameBadArgs.html and write tests to assert that invalid inputs throw exceptions.
    3. If your test causes a segfault, add the following to the top of the script tag: <code>Tests.autorun = false; Tests.message = "Caution: this may crash your browser";</code>
  4. For each performance test:
    1. Write a performance/myTestName.html and set <code>Tests.autorun = false;</code>
  5. If you have a test that you would like to run over the whole API or want to generate tests programmatically, add them to gen_tests.rb or write your own script.
  6. Create a commit for each file. (E.g. <code>for f in $(git status | grep -e "^#\\s*functions/\\S*$" | sed "s/^#\s*//"); do git add $f; git commit -m $f; done</code>)
  7. Send me a pull request.
  8. Congratulations, you're now a contributor!


For more information on WebGL:

  * <a href="http://planet-webgl.org">Planet WebGL</a>
  * <a href="http://learningwebgl.com">Learning WebGL</a>
  * <a href="http://www.khronos.org/message_boards/viewforum.php?f=34">WebGL on Khronos Message Boards</a>

Developer links:

  * <a href="https://bugzilla.mozilla.org/buglist.cgi?quicksearch=webgl">WebGL on Mozilla Bugzilla</a>
  * <a href="https://bugzilla.webkit.org/buglist.cgi?quicksearch=webgl">WebGL on WebKit Bugzilla</a>
  * <a href="http://code.google.com/p/chromium/issues/list?q=label:3D-WebGL">WebGL on Chromium Bugzilla</a>

What's the stuff in apigen?

  There are some Python scripts in the apigen/ directory that generate C++ based on the API definition files (gl2.h, api_modifications.txt, valid_args.txt.) The generated code is Mozilla XPCOM functions that check their args against the valid GLES 2.0 constants (as they were written on the man pages.) There's also some wackier stuff for checking copyTexImage2D and copyTexSubImage2D image dimensions against viewport dimensions.

  If you can use it to generate code for your WebGL implementation, it might save you 1500 lines of typing and testing. The last time I used it was summer 2009 to generate a patch for Canvas 3D, so it's likely somewhat out of date.
