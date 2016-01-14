/*
** Copyright (c) 2012 The Khronos Group Inc.
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

// This is a test harness for running javascript tests in the browser.
// The only identifier exposed by this harness is WebGLTestHarnessModule.
//
// To use it make an HTML page with an iframe. Then call the harness like this
//
//    function reportResults(type, msg, success) {
//      ...
//      return true;
//    }
//
//    var fileListURL = '00_test_list.txt';
//    var testHarness = new WebGLTestHarnessModule.TestHarness(
//        iframe,
//        fileListURL,
//        reportResults,
//        options);
//
// The harness will load the fileListURL and parse it for the URLs, one URL
// per line preceded by options, see below. URLs should be on the same domain
// and at the  same folder level or below the main html file.  If any URL ends
// in .txt it will be parsed as well so you can nest .txt files. URLs inside a
// .txt file should be relative to that text file.
//
// During startup, for each page found the reportFunction will be called with
// WebGLTestHarnessModule.TestHarness.reportType.ADD_PAGE and msg will be
// the URL of the test.
//
// Each test is required to call testHarness.reportResults. This is most easily
// accomplished by storing that value on the main window with
//
//     window.webglTestHarness = testHarness
//
// and then adding these to functions to your tests.
//
//     function reportTestResultsToHarness(success, msg) {
//       if (window.parent.webglTestHarness) {
//         window.parent.webglTestHarness.reportResults(success, msg);
//       }
//     }
//
//     function notifyFinishedToHarness() {
//       if (window.parent.webglTestHarness) {
//         window.parent.webglTestHarness.notifyFinished();
//       }
//     }
//
// This way your tests will still run without the harness and you can use
// any testing framework you want.
//
// Each test should call reportTestResultsToHarness with true for success if it
// succeeded and false if it fail followed and any message it wants to
// associate with the test. If your testing framework supports checking for
// timeout you can call it with success equal to undefined in that case.
//
// To run the tests, call testHarness.runTests(options);
//
// For each test run, before the page is loaded the reportFunction will be
// called with WebGLTestHarnessModule.TestHarness.reportType.START_PAGE and msg
// will be the URL of the test. You may return false if you want the test to be
// skipped.
//
// For each test completed the reportFunction will be called with
// with WebGLTestHarnessModule.TestHarness.reportType.TEST_RESULT,
// success = true on success, false on failure, undefined on timeout
// and msg is any message the test choose to pass on.
//
// When all the tests on the page have finished your page must call
// notifyFinishedToHarness.  If notifyFinishedToHarness is not called
// the harness will assume the test timed out.
//
// When all the tests on a page have finished OR the page as timed out the
// reportFunction will be called with
// WebGLTestHarnessModule.TestHarness.reportType.FINISH_PAGE
// where success = true if the page has completed or undefined if the page timed
// out.
//
// Finally, when all the tests have completed the reportFunction will be called
// with WebGLTestHarnessModule.TestHarness.reportType.FINISHED_ALL_TESTS.
//
// Harness Options
//
// These are passed in to the TestHarness as a JavaScript object
//
// version: (required!)
//
//     Specifies a version used to filter tests. Tests marked as requiring
//     a version greater than this version will not be included.
//
//     example: new TestHarness(...., {version: "3.1.2"});
//
// minVersion:
//
//     Specifies the minimum version a test must require to be included.
//     This basically flips the filter so that only tests marked with
//     --min-version will be included if they are at this minVersion or
//     greater.
//
//     example: new TestHarness(...., {minVersion: "2.3.1"});
//
// maxVersion:
//
//     Specifies the maximum version a test must require to be included.
//     This basically flips the filter so that only tests marked with
//     --max-version will be included if they are at this maxVersion or
//     less.
//
//     example: new TestHarness(...., {maxVersion: "2.3.1"});
//
// fast:
//
//     Specifies to skip any tests marked as slow.
//
//     example: new TestHarness(..., {fast: true});
//
// Test Options:
//
// Any test URL or .txt file can be prefixed by the following options
//
// min-version:
//
//     Sets the minimum version required to include this test. A version is
//     passed into the harness options. Any test marked as requiring a
//     min-version greater than the version passed to the harness is skipped.
//     This allows you to add new tests to a suite of tests for a future
//     version of the suite without including the test in the current version.
//     If no -min-version is specified it is inheriited from the .txt file
//     including it. The default is 1.0.0
//
//     example:  --min-version 2.1.3 sometest.html
//
// max-version:
//
//     Sets the maximum version required to include this test. A version is
//     passed into the harness options. Any test marked as requiring a
//     max-version less than the version passed to the harness is skipped.
//     This allows you to test functionality that has been removed from later
//     versions of the suite.
//     If no -max-version is specified it is inherited from the .txt file
//     including it.
//
//     example:  --max-version 1.9.9 sometest.html
//
// slow:
//
//     Marks a test as slow. Slow tests can be skipped by passing fastOnly: true
//     to the TestHarness. Of course you need to pass all tests but sometimes
//     you'd like to test quickly and run only the fast subset of tests.
//
//     example:  --slow some-test-that-takes-2-mins.html
//

WebGLTestHarnessModule = function() {

/**
 * Wrapped logging function.
 */
var log = function(msg) {
  if (window.console && window.console.log) {
    window.console.log(msg);
  }
};

/**
 * Loads text from an external file. This function is synchronous.
 * @param {string} url The url of the external file.
 * @param {!function(bool, string): void} callback that is sent a bool for
 *     success and the string.
 */
var loadTextFileAsynchronous = function(url, callback) {
  log ("loading: " + url);
  var error = 'loadTextFileSynchronous failed to load url "' + url + '"';
  var request;
  if (window.XMLHttpRequest) {
    request = new XMLHttpRequest();
    if (request.overrideMimeType) {
      request.overrideMimeType('text/plain');
    }
  } else {
    throw 'XMLHttpRequest is disabled';
  }
  try {
    request.open('GET', url, true);
    request.onreadystatechange = function() {
      if (request.readyState == 4) {
        var text = '';
        // HTTP reports success with a 200 status. The file protocol reports
        // success with zero. HTTP does not use zero as a status code (they
        // start at 100).
        // https://developer.mozilla.org/En/Using_XMLHttpRequest
        var success = request.status == 200 || request.status == 0;
        if (success) {
          text = request.responseText;
        }
        log("loaded: " + url);
        callback(success, text);
      }
    };
    request.send(null);
  } catch (e) {
    log("failed to load: " + url);
    callback(false, '');
  }
};

/**
 * @param {string} versionString WebGL version string.
 * @return {number} Integer containing the WebGL major version.
 */
var getMajorVersion = function(versionString) {
  if (!versionString) {
    return 1;
  }
  return parseInt(versionString.split(" ")[0].split(".")[0], 10);
};

/**
 * @param {string} url Base URL of the test.
 * @param {number} webglVersion Integer containing the WebGL major version.
 * @param {boolean} dumpShaders add dumpShader query parameter if true.
 * @return {string} URL that will run the test with the given WebGL version.
 */
var getURLWithOptions = function(url, webglVersion, dumpShaders) {
  var queryArgs = 0;

  if (webglVersion) {
    url += "?webglVersion=" + webglVersion;
    queryArgs++;
  }

  if (dumpShaders) {
    url += queryArgs ? "&" : "?";
    url += "dumpShaders=1";
  }

  return url;
};

/**
 * Compare version strings.
 */
var greaterThanOrEqualToVersion = function(have, want) {
  have = have.split(" ")[0].split(".");
  want = want.split(" ")[0].split(".");

  //have 1.2.3   want  1.1
  //have 1.1.1   want  1.1
  //have 1.0.9   want  1.1
  //have 1.1     want  1.1.1

  for (var ii = 0; ii < want.length; ++ii) {
    var wantNum = parseInt(want[ii]);
    var haveNum = have[ii] ? parseInt(have[ii]) : 0
    if (haveNum > wantNum) {
      return true; // 2.0.0 is greater than 1.2.3
    }
    if (haveNum < wantNum) {
      return false;
    }
  }
  return true;
};

/**
 * Reads a file, recursively adding files referenced inside.
 *
 * Each line of URL is parsed, comments starting with '#' or ';'
 * or '//' are stripped.
 *
 * arguments beginning with -- are extracted
 *
 * lines that end in .txt are recursively scanned for more files
 * other lines are added to the list of files.
 *
 * @param {string} url The url of the file to read.
 * @param {void function(boolean, !Array.<string>)} callback.
 *      Callback that is called with true for success and an
 *      array of filenames.
 * @param {Object} options. Optional options
 *
 * Options:
 *    version: {string} The version of the conformance test.
 *    Tests with the argument --min-version <version> will
 *    be ignored version is less then <version>
 *
 */
var getFileList = function(url, callback, options) {
  var files = [];

  var copyObject = function(obj) {
    return JSON.parse(JSON.stringify(obj));
  };

  var toCamelCase = function(str) {
    return str.replace(/-([a-z])/g, function (g) { return g[1].toUpperCase() });
  };

  var globalOptions = copyObject(options);
  globalOptions.defaultVersion = "1.0";
  globalOptions.defaultMaxVersion = null;

  var getFileListImpl = function(prefix, line, lineNum, hierarchicalOptions, callback) {
    var files = [];

    var args = line.split(/\s+/);
    var nonOptions = [];
    var useTest = true;
    var testOptions = {};
    for (var jj = 0; jj < args.length; ++jj) {
      var arg = args[jj];
      if (arg[0] == '-') {
        if (arg[1] != '-') {
          throw ("bad option at in " + url + ":" + lineNum + ": " + arg);
        }
        var option = arg.substring(2);
        switch (option) {
          // no argument options.
          case 'slow':
            testOptions[toCamelCase(option)] = true;
            break;
          // one argument options.
          case 'min-version':
          case 'max-version':
            ++jj;
            testOptions[toCamelCase(option)] = args[jj];
            break;
          default:
            throw ("bad unknown option '" + option + "' at in " + url + ":" + lineNum + ": " + arg);
        }
      } else {
        nonOptions.push(arg);
      }
    }
    var url = prefix + nonOptions.join(" ");

    if (url.substr(url.length - 4) != '.txt') {
      var minVersion = testOptions.minVersion;
      if (!minVersion) {
        minVersion = hierarchicalOptions.defaultVersion;
      }
      var maxVersion = testOptions.maxVersion;
      if (!maxVersion) {
        maxVersion = hierarchicalOptions.defaultMaxVersion;
      }
      var slow = testOptions.slow;
      if (!slow) {
        slow = hierarchicalOptions.defaultSlow;
      }

      if (globalOptions.fast && slow) {
        useTest = false;
      } else if (globalOptions.minVersion) {
        useTest = greaterThanOrEqualToVersion(minVersion, globalOptions.minVersion);
      } else if (globalOptions.maxVersion && maxVersion) {
        useTest = greaterThanOrEqualToVersion(globalOptions.maxVersion, maxVersion);
      } else {
        useTest = greaterThanOrEqualToVersion(globalOptions.version, minVersion);
        if (maxVersion) {
          useTest = useTest && greaterThanOrEqualToVersion(maxVersion, globalOptions.version);
        }
      }
    }

    if (!useTest) {
      callback(true, []);
      return;
    }

    if (url.substr(url.length - 4) == '.txt') {
      // If a version was explicity specified pass it down.
      if (testOptions.minVersion) {
        hierarchicalOptions.defaultVersion = testOptions.minVersion;
      }
      if (testOptions.maxVersion) {
        hierarchicalOptions.defaultMaxVersion = testOptions.maxVersion;
      }
      if (testOptions.slow) {
        hierarchicalOptions.defaultSlow = testOptions.slow;
      }
      loadTextFileAsynchronous(url, function() {
        return function(success, text) {
          if (!success) {
            callback(false, '');
            return;
          }
          var lines = text.split('\n');
          var prefix = '';
          var lastSlash = url.lastIndexOf('/');
          if (lastSlash >= 0) {
            prefix = url.substr(0, lastSlash + 1);
          }
          var fail = false;
          var count = 1;
          var index = 0;
          for (var ii = 0; ii < lines.length; ++ii) {
            var str = lines[ii].replace(/^\s\s*/, '').replace(/\s\s*$/, '');
            if (str.length > 4 &&
                str[0] != '#' &&
                str[0] != ";" &&
                str.substr(0, 2) != "//") {
              ++count;
              getFileListImpl(prefix, str, ii + 1, copyObject(hierarchicalOptions), function(index) {
                return function(success, new_files) {
                  //log("got files: " + new_files.length);
                  if (success) {
                    files[index] = new_files;
                  }
                  finish(success);
                };
              }(index++));
            }
          }
          finish(true);

          function finish(success) {
            if (!success) {
              fail = true;
            }
            --count;
            //log("count: " + count);
            if (!count) {
              callback(!fail, files);
            }
          }
        }
      }());
    } else {
      files.push(url);
      callback(true, files);
    }
  };

  getFileListImpl('', url, 1, globalOptions, function(success, files) {
    // flatten
    var flat = [];
    flatten(files);
    function flatten(files) {
      for (var ii = 0; ii < files.length; ++ii) {
        var value = files[ii];
        if (typeof(value) == "string") {
          flat.push(value);
        } else {
          flatten(value);
        }
      }
    }
    callback(success, flat);
  });
};

var FilterURL = (function() {
  var prefix = window.location.pathname;
  prefix = prefix.substring(0, prefix.lastIndexOf("/") + 1);
  return function(url) {
    if (url.substring(0, prefix.length) == prefix) {
      url = url.substring(prefix.length);
    }
    return url;
  };
}());

var TestFile = function(url) {
  this.url = url;
};

var Test = function(file) {
  this.file = file;
};

var TestHarness = function(iframe, filelistUrl, reportFunc, options) {
  this.window = window;
  this.iframes = iframe.length ? iframe : [iframe];
  this.reportFunc = reportFunc;
  this.timeoutDelay = 20000;
  this.files = [];
  this.allowSkip = options.allowSkip;
  this.webglVersion = getMajorVersion(options.version);
  this.dumpShaders = options.dumpShaders;

  var that = this;
  getFileList(filelistUrl, function() {
    return function(success, files) {
      that.addFiles_(success, files);
    };
  }(), options);

};

TestHarness.reportType = {
  ADD_PAGE: 1,
  READY: 2,
  START_PAGE: 3,
  TEST_RESULT: 4,
  FINISH_PAGE: 5,
  FINISHED_ALL_TESTS: 6
};

TestHarness.prototype.addFiles_ = function(success, files) {
  if (!success) {
    this.reportFunc(
        TestHarness.reportType.FINISHED_ALL_TESTS,
        '',
        'Unable to load tests. Are you running locally?\n' +
        'You need to run from a server or configure your\n' +
        'browser to allow access to local files (not recommended).\n\n' +
        'Note: An easy way to run from a server:\n\n' +
        '\tcd path_to_tests\n' +
        '\tpython -m SimpleHTTPServer\n\n' +
        'then point your browser to ' +
          '<a href="http://localhost:8000/webgl-conformance-tests.html">' +
          'http://localhost:8000/webgl-conformance-tests.html</a>',
        false)
    return;
  }
  log("total files: " + files.length);
  for (var ii = 0; ii < files.length; ++ii) {
    log("" + ii + ": " + files[ii]);
    this.files.push(new TestFile(files[ii]));
    this.reportFunc(TestHarness.reportType.ADD_PAGE, '', files[ii], undefined);
  }
  this.reportFunc(TestHarness.reportType.READY, '', undefined, undefined);
}

TestHarness.prototype.runTests = function(opt_options) {
  var options = opt_options || { };
  options.start = options.start || 0;
  options.count = options.count || this.files.length;

  this.idleIFrames = this.iframes.slice(0);
  this.runningTests = {};
  var testsToRun = [];
  for (var ii = 0; ii < options.count; ++ii) {
    testsToRun.push(ii + options.start);
  }
  this.numTestsRemaining = options.count;
  this.testsToRun = testsToRun;
  this.startNextTest();
};

TestHarness.prototype.setTimeout = function(test) {
  var that = this;
  test.timeoutId = this.window.setTimeout(function() {
      that.timeout(test);
    }, this.timeoutDelay);
};

TestHarness.prototype.clearTimeout = function(test) {
  this.window.clearTimeout(test.timeoutId);
};

TestHarness.prototype.startNextTest = function() {
  if (this.numTestsRemaining == 0) {
    log("done");
    this.reportFunc(TestHarness.reportType.FINISHED_ALL_TESTS,
                    '', '', true);
  } else {
    while (this.testsToRun.length > 0 && this.idleIFrames.length > 0) {
      var testId = this.testsToRun.shift();
      var iframe = this.idleIFrames.shift();
      this.startTest(iframe, this.files[testId], this.webglVersion);
    }
  }
};

TestHarness.prototype.startTest = function(iframe, testFile, webglVersion) {
  var test = {
    iframe: iframe,
    testFile: testFile
  };
  var url = testFile.url;
  this.runningTests[url] = test;
  log("loading: " + url);
  if (this.reportFunc(TestHarness.reportType.START_PAGE, url, url, undefined)) {
    if (this.dumpShaders == 1)
      iframe.src = getURLWithOptions(url, webglVersion, true);
    else
      iframe.src = getURLWithOptions(url, webglVersion);
    this.setTimeout(test);
  } else {
    this.reportResults(url, !!this.allowSkip, "skipped", true);
    this.notifyFinished(url);
  }
};

TestHarness.prototype.getTest = function(url) {
  var test = this.runningTests[FilterURL(url)];
  if (!test) {
    throw("unknown test:" + url);
  }
  return test;
};

TestHarness.prototype.reportResults = function(url, success, msg, skipped) {
  url = FilterURL(url);
  var test = this.getTest(url);
  this.clearTimeout(test);
  log(success ? "PASS" : "FAIL", msg);
  this.reportFunc(TestHarness.reportType.TEST_RESULT, url, msg, success, skipped);
  // For each result we get, reset the timeout
  this.setTimeout(test);
};

TestHarness.prototype.dequeTest = function(test) {
  this.clearTimeout(test);
  this.idleIFrames.push(test.iframe);
  delete this.runningTests[test.testFile.url];
  --this.numTestsRemaining;
}

TestHarness.prototype.notifyFinished = function(url) {
  url = FilterURL(url);
  var test = this.getTest(url);
  log(url + ": finished");
  this.dequeTest(test);
  this.reportFunc(TestHarness.reportType.FINISH_PAGE, url, url, true);
  this.startNextTest();
};

TestHarness.prototype.timeout = function(test) {
  this.dequeTest(test);
  var url = test.testFile.url;
  log(url + ": timeout");
  this.reportFunc(TestHarness.reportType.FINISH_PAGE, url, url, undefined);
  this.startNextTest();
};

TestHarness.prototype.setTimeoutDelay = function(x) {
  this.timeoutDelay = x;
};

return {
    'TestHarness': TestHarness,
    'getMajorVersion': getMajorVersion,
    'getURLWithOptions': getURLWithOptions
  };

}();



