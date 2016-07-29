/* globals add_completion_callback, Promise, done, assert_true, Ajv, on_event */

/**
 * Creates a JSONtest object.  If the parameters are supplied
 * it also loads a referenced testFile, processes that file, loads any
 * referenced external assertions, and sets up event listeners to process the
 * user's test data.  The loading is done asynchronously via Promises.  The test
 * button's text is changed to Loading while it is processing, and to "Check
 * JSON" once the data is loaded.
 *
 * @constructor
 * @param {object} params
 * @param {string} [params.test] - object containing JSON test definition
 * @param {string} [params.testFile] - URI of a file with JSON test definition
 * @param {string} params.runTest - IDREF of an element that when clicked will run the test
 * @param {string} params.testInput - IDREF of an element that contains the JSON(-LD) to evaluate against the assertions in the test / testFile
 * @event DOMContentLoaded Calls init once DOM is fully loaded
 * @returns {object} Reference to the new object
 */

function JSONtest(params) {
  'use strict';

  this.Assertions = [];     // object that will contain the assertions to process
  this.AssertionText = "";  // string that holds the titles of all the assertions in use
  this.DescriptionText = "";
  this.Base = null;         // URI "base" for the test suite being run
  this.TestDir = null;      // URI "base" for the test case being run
  this.Params = null;       // paramaters passed in
  this.Properties = null;   // testharness_properties from the opening window
  this.Test = null;         // test being run
  this.AssertionCounter = 0;// keeps track of which assertion is being processed

  this._assertionText = []; // Array of text or nested arrays of assertions
  this._assertionCache = [];// Array to put loaded assertions into
  this._loading = true;

  var pending = [] ;

  // set up in case DOM finishes loading early
  pending.push(new Promise(function(resolve) {
    on_event(document, "DOMContentLoaded", function() {
        resolve(true);
    }.bind(this));
  }.bind(this)));

  // create an ajv object that will stay around so that caching
  // of schema that are compiled just works
  this.ajv = new Ajv({allErrors: true, validateSchema: false}) ;

  // determine the base URI for the test collection.  This is
  // the top level folder in the test "document.location"

  var l = document.location;
  var p = l.pathname;
  this.TestDir = p.substr(0, 1+p.lastIndexOf('/'));
  this.Base = p.substr(0, 1+p.indexOf('/', 1));

  // if we are under runner, then there are props in the parent window
  //
  // if "output" is set in that, then pause at the end of running so the output
  // can be analyzed. @@@TODO@@@
  if (window && window.opener && window.opener.testharness_properties) {
    this.Properties = window.opener.testharness_properties;
  }

  this.Params = params;

  // if there is a list of definitions in the params,
  // include them
  if (this.Params.schemaDefs) {
    var defPromise = new Promise(function(resolve, reject) {
      var promisedSchema = this.Params.schemaDefs.map(function(item) {
        return this.loadDefinition(item);
      }.bind(this));

      // Once all the loadAssertion promises resolve...
      Promise.all(promisedSchema)
      .then(function (schemaContents) {
        this.ajv.addSchema(schemaContents);
        resolve(true);
      }.bind(this))
      .catch(function(err) {
        reject(err);
      }.bind(this));
    }.bind(this));
    // these schema need to load up too
    pending.push(defPromise) ;
  }

  // start by loading the test (it might be inline, but
  // loadTest deals with that
  pending.push(this.loadTest(params)
    .then(function(test) {
      // if the test is NOT an object, turn it into one
      if (typeof test === 'string') {
        test = JSON.parse(test) ;
      }

      this.Test = test;

      // Test should have information that we can put in the template

      if (test.description) {
        this.DescriptionText = test.description;
      }

      return new Promise(function(resolve, reject) {
        if (test.assertions &&
            typeof test.assertions === "object") {
          // we have at least one assertion
          // get the inline contents and the references to external files
          var assertFiles = this._assertionRefs(test.assertions);

          var promisedAsserts = assertFiles.map(function(item) {
            return this.loadAssertion(item);
          }.bind(this));

          // Once all the loadAssertion promises resolve...
          Promise.all(promisedAsserts)
          .then(function (assertContents) {
            // assertContents has assertions in document order

            var assertIdx = 0;

            // populate the display of assertions that are being exercised
            // returns the list of top level assertions to walk through

            var buildList = function(assertions, level) {
              if (level === undefined) {
                level = 1;
              }

              // accumulate the assertions - but only when level is 0
              var list = [] ;

              if (assertions) {
                if (typeof assertions === "object" && assertions.hasOwnProperty('assertions')) {
                  // this is a conditionObject
                  if (level === 0) {
                    list.push(assertContents[assertIdx]);
                  }

                  this.AssertionText += "<li>" + assertContents[assertIdx++].title;
                  this.AssertionText += "<ol>";
                  buildList(assertions.assertions, level+1) ;
                  this.AssertionText += "</ol></li>\n";
                } else {
                  // it is NOT a conditionObject - must be an array
                  assertions.forEach( function(assert) {
                    if (typeof assert === "object" && Array.isArray(assert)) {
                      this.AssertionText += "<ol>";
                      // it is a nested list - recurse
                      buildList(assert, level+1) ;
                      this.AssertionText += "</ol>\n";
                    } else if (typeof assert === "object" && !Array.isArray(assert) && assert.hasOwnProperty('assertions')) {
                      if (level === 0) {
                        list.push(assertContents[assertIdx]);
                      }
                      // there is a condition object in the array
                      this.AssertionText += "<li>" + assertContents[assertIdx++].title;
                      this.AssertionText += "<ol>";
                      buildList(assert, level+1) ; // capture the children too
                      this.AssertionText += "</ol></li>\n";
                    } else {
                      if (level === 0) {
                        list.push(assertContents[assertIdx]);
                      }
                      this.AssertionText += "<li>" + assertContents[assertIdx++].title + "</li>\n";
                    }
                  }.bind(this));
                }
              }
              return list;
            }.bind(this);

            // Assertions will ONLY contain the top level assertions
            this.Assertions = buildList(test.assertions, 0);
            resolve(true);
          }.bind(this))
          .catch(function(err) {
            reject(err);
          }.bind(this));
        } else {
          if (!test.assertions) {
            reject("Test has no assertion property");
          } else {
            reject("Test assertion property is not an Array");
          }
        }
      }.bind(this));
    }.bind(this)));

  // once the DOM and the test / assertions are loaded... set us up
  Promise.all(pending)
  .then(function() {
    this.loading = false;
    this.init();
  }.bind(this))
  .catch(function(err) {
    // loading the components failed somehow - report the errors and mark the test failed
    test( function() {
      assert_true(false, "Loading of test components failed: " +JSON.stringify(err)) ;
    }, "Loading test components");
    done() ;
    return ;
  }.bind(this));

  return this;
}

JSONtest.prototype = {

  /**
   * @listens click
   */
  init: function() {
    'use strict';
    // set up a handler
    var runButton = document.getElementById(this.Params.runTest) ;
    var closeButton = document.getElementById(this.Params.closeWindow) ;
    var testInput  = document.getElementById(this.Params.testInput) ;
    var assertion  = document.getElementById("assertion") ;
    var desc  = document.getElementById("testDescription") ;

    if (!this.loading) {
      runButton.disabled = false;
      runButton.value = "Check JSON";
      if (desc) {
        desc.innerHTML = this.DescriptionText;
      }
      if (assertion) {
        assertion.innerHTML = "<ol>" + this.AssertionText + "</ol>\n";
      }
    } else {
      window.alert("Loading did not finish before init handler was called!");
    }

    // @@@TODO@@@ implement the output showing handler
    if (0 && this.Properties && this.Properties.output && closeButton) {
      // set up a callback
      add_completion_callback( function() {
        var p = new Promise(function(resolve) {
          closeButton.style.display = "inline";
          closeButton.disabled = false;
          on_event(closeButton, "click", function() {
            resolve(true);
          });
        }.bind(this));
        p.then();
      }.bind(this));
    }

    on_event(runButton, "click", function() {
      // user clicked
      var content = testInput.value;
      runButton.disabled = true;

      // make sure content is an object
      if (typeof content === "string") {
        try {
          content = JSON.parse(content) ;
        } catch(err) {
          // if the parsing failed, create a special test and mark it failed
          test( function() {
            assert_true(false, "Parse of JSON failed: " + err) ;
          }, "Parsing submitted input");
          // and just give up
          done();
          return ;
        }
      }

      // iterate over all of the tests for this instance
      this.runTests(this.Assertions, content);

      // explicitly tell the test framework we are done
      done();
    }.bind(this));
  },

  // runTests - process tests
  /**
   * @param {object} assertions - List of assertions to process
   * @param {string} content - JSON(-LD) to be evaluated
   * @param {string} [testAction='continue'] - state of test processing (in parent when recursing)
   * @param {integer} [level=0] - depth of recursion since assertion lists can nest
   * @param {string} [compareWith='and'] - the way the results of the referenced assertions should be compared
   * @returns {string} - the testAction resulting from evaluating all of the assertions
   */
  runTests: function(assertions, content, testAction, level, compareWith) {
    'use strict';

    // level
    if (level === undefined) {
      level = 1;
    }

    // testAction
    if (testAction === undefined) {
      testAction = 'continue';
    }

    // compareWith
    if (compareWith === undefined) {
      compareWith = 'and';
    }

    // for each assertion (in order) load the external json schema if
    // one is referenced, or use the inline schema if supplied
    // validate content against the referenced schema

    var theResults = [] ;

    if (assertions) {

      assertions.forEach( function(assert, num) {

        var expected = assert.hasOwnProperty('expectedResult') ? assert.expectedResult : 'valid' ;
        var message = assert.hasOwnProperty('message') ? assert.message : "Result was not " + expected;

        // first - what is the type of the assert
        if (typeof assert === "object" && !Array.isArray(assert)) {
          if (assert.hasOwnProperty("compareWith") && assert.hasOwnProperty("assertions") && Array.isArray(assert.assertions) ) {
            // this is a comparisonObject
            var r = this.runTests(assert.assertions, content, testAction, level+1, assert.compareWith);
            // r is an object that contains, among other things, an array of results from the child assertions
            testAction = r.action;

            // evaluate the results against the compareWith setting
            var result = true;
            var data = r.results ;
            var i;

            if (assert.compareWith === "or") {
              result = false;
              for(i = 0; i < data.length; i++) {
                if (data[i]) {
                  result = true;
                }
              }
            } else {
              for(i = 0; i < data.length; i++) {
                if (!data[i]) {
                  result = false;
                }
              }
            }

            // create a test and push the result
            test(function() {
              var newAction = this.determineAction(assert, result) ;
              // next time around we will use this action
              testAction = newAction;

              var err = ";";

              if (testAction === 'abort') {
                err += "; Aborting execution of remaining assertions;";
              } else if (testAction === 'skip') {
                err += "; Skipping execution of remaining assertions at level " + level + ";";
              }

              if (result === false) {
                // test result was unexpected; use message
                assert_true(result, message + err);
              } else {
                assert_true(result, err) ;
              }
            }.bind(this), "" + level + ":" + (num+1) + " " + assert.title);
            // we are going to return out of this
            return;
          }
        } else if (typeof assert === "object" && Array.isArray(assert)) {
          // it is a nested list - recurse
          var o = this.runTests(assert, content, testAction, level+1);
          if (o.result && o.result === 'abort') {
            // we are bailing out
            testAction = 'abort';
          }
        }

        if (testAction === 'abort') {
          return {action: 'abort' };
        }

        var schemaName = "inline " + level + ":" + (num+1);

        if (typeof assert === "string") {
          // the assertion passed in is a file name; find it in the cache
          if (this._assertionCache[assert]) {
            assert = this._assertionCache[assert];
          } else {
            test( function() {
              assert_true(false, "Reference to assertion " + assert + " at level " + level + ":" + (num+1) + " unresolved") ;
            }, "Processing " + assert);
            return ;
          }
        }

        if (assert.assertionFile) {
          schemaName = "external file " + assert.assertionFile + " " + level + ":" + (num+1);
        }

        var validate = null;

        try {
          validate = this.ajv.compile(assert);
        }
        catch(err) {
          test( function() {
            assert_true(false, "Compilation of schema " + level + ":" + (num+1) + " failed: " + err) ;
          }, "Compiling " + schemaName);
          return ;
        }

        if (testAction !== 'continue') {
          // a previous test told us to not run this test; skip it
          test(function() { }, "SKIPPED: " + assert.title);
        } else {
          // start an actual sub-test
          test(function() {
            var valid = validate(content) ;

            var result = this.determineResult(assert, valid) ;

            // remember the result
            theResults.push(result);

            var newAction = this.determineAction(assert, result) ;
            // next time around we will use this action
            testAction = newAction;

            var err = ";";
            if (validate.errors !== null) {
              err = "; Errors: " + this.ajv.errorsText(validate.errors) + ";" ;
            }
            if (testAction === 'abort') {
              err += "; Aborting execution of remaining assertions;";
            } else if (testAction === 'skip') {
              err += "; Skipping execution of remaining assertions at level " + level + ";";
            }
            if (result === false) {
              // test result was unexpected; use message
              assert_true(result, message + err);
            } else {
              assert_true(result, err) ;
            }
          }.bind(this), "" + level + ":" + (num+1) + " " + assert.title);
        }
      }.bind(this));
    }

    return { action: testAction, results: theResults} ;
  },

  determineResult: function(schema, valid) {
    'use strict';
    var r = 'valid' ;
    if (schema.hasOwnProperty('expectedResult')) {
      r = schema.expectedResult;
    }

    if (r === 'valid' && valid || r === 'invalid' && !valid) {
      return true;
    } else {
      return false;
    }
  },

  determineAction: function(schema, result) {
    'use strict';
    // mapping from results to actions
    var mapping = {
      'failAndContinue' : 'continue',
      'failAndSkip'    : 'skip',
      'failAndAbort'   : 'abort',
      'passAndContinue': 'continue',
      'passAndSkip'    : 'skip',
      'passAndAbort'   : 'abort'
    };

    // if the result was as expected, then just keep going
    if (result) {
      return 'continue';
    }

    var a = 'failAndContinue';

    if (schema.hasOwnProperty('onUnexpectedResult')) {
      a = schema.onUnexpectedResult;
    }

    if (mapping[a]) {
      return mapping[a];
    } else {
      return 'continue';
    }
  },

  // loadAssertion - load an Assertion from an external JSON file
  //
  // returns a promise that resolves with the contents of the assertion file

  loadAssertion: function(afile) {
    'use strict';
    if (typeof(afile) === 'string') {
      var theFile = this._parseURI(afile);
      // it is a file reference - load it
      return new Promise(function(resolve, reject) {
        this._loadFile("GET", theFile, true)
          .then(function(data) {
            data.assertionFile = afile;
            this._assertionCache[afile] = data;
            resolve(data);
          }.bind(this))
          .catch(function(err) {
            if (typeof err === "object") {
              err.theFile = theFile;
            }
            reject(err);
          });
        }.bind(this));
      }
      else if (afile.hasOwnProperty("assertionFile")) {
      // this object is referecing an external assertion
      return new Promise(function(resolve, reject) {
        var theFile = this._parseURI(afile.assertionFile);
        this._loadFile("GET", theFile, true)
        .then(function(external) {
          // okay - we have an external object
          Object.keys(afile).forEach(function(key) {
            if (key !== 'assertionFile') {
              external[key] = afile[key];
            }
          });
          resolve(external);
        }.bind(this))
        .catch(function(err) {
          if (typeof err === "object") {
            err.theFile = theFile;
          }
          reject(err);
        });
      }.bind(this));
    } else {
      // it is already a loaded assertion - just use it
      return new Promise(function(resolve) {
        resolve(afile);
      });
    }
  },

  // loadDefinition - load a JSON Schema definition from an external JSON file
  //
  // returns a promise that resolves with the contents of the definition file

  loadDefinition: function(dfile) {
    'use strict';
    return new Promise(function(resolve, reject) {
      this._loadFile("GET", this._parseURI(dfile), true)
        .then(function(data) {
          resolve(data);
        }.bind(this))
        .catch(function(err) {
          reject(err);
        });
      }.bind(this));
  },


  // loadTest - load a test from an external JSON file
  //
  // returns a promise that resolves with the contents of the
  // test

  loadTest: function(params) {
    'use strict';

    if (params.hasOwnProperty('testFile')) {
      // the test is referred to by a file name
      return this._loadFile("GET", params.testFile);
    } // else
    return new Promise(function(resolve, reject) {
      if (params.hasOwnProperty('test')) {
        resolve(params.test);
      } else {
        reject("Must supply a 'test' or 'testFile' parameter");
      }
    });
  },

  _parseURI: function(theURI) {
    'use strict';
    // determine what the top level URI should be
    if (theURI.indexOf('/') === -1) {
      // no slash - it's relative to where we are
      // so just use it
      return this.TestDir + theURI;
    } else if (theURI.indexOf('/') === 0 || theURI.indexOf('http:') === 0 || theURI.indexOf('https:') === 0) {
      // it is an absolute URI so just use it
      return theURI;
    } else {
      // it is relative and contains a slash.
      // make it relative to the current test root
      return this.Base + theURI;
    }
  },

  /**
   * return a list of all inline assertions or references
   *
   * @param {array} assertions list of assertions to examine
   */

  _assertionRefs: function(assertions) {
    'use strict';
    var ret = [] ;

    // when the reference is to an object that has an array of assertions in it (a conditionObject)
    // then remember that one and loop over its embedded assertions
    if (typeof(assertions) === "object" && !Array.isArray(assertions) && assertions.hasOwnProperty('assertions')) {
      ret.push(assertions) ;
      assertions = assertions.assertions;
    }
    if (typeof(assertions) === "object" && Array.isArray(assertions)) {
      assertions.forEach( function(assert) {
        // first - what is the type of the assert
        if (typeof assert === "object" && Array.isArray(assert)) {
          // it is a nested list - recurse
          this._assertionRefs(assert).forEach( function(item) {
            ret.push(item);
          }.bind(this));
        } else if (typeof assert === "object") {
          ret.push(assert) ;
          if (assert.hasOwnProperty("assertions")) {
            // there are embedded assertions; get those too
            ret.concat(this._assertionRefs(assert.assertions));
          }
        } else {
          // it is a file name
          ret.push(assert) ;
        }
      }.bind(this));
    }
    return ret;
  },

  // _loadFile - return a promise loading a file
  //
  _loadFile: function(method, url, parse) {
    'use strict';
    if (parse === undefined) {
      parse = true;
    }

    return new Promise(function (resolve, reject) {
      if (document.location.search) {
        var s = document.location.search;
        s = s.replace(/^\?/, '');
        if (url.indexOf('?') !== -1) {
          url += "&" + s;
        } else {
          url += "?" + s;
        }
      }
      var xhr = new XMLHttpRequest();
      xhr.open(method, url);
      xhr.onload = function () {
        if (this.status >= 200 && this.status < 300) {
          var d = xhr.response;
          if (parse) {
            try {
              d = JSON.parse(d);
              resolve(d);
            }
            catch(err) {
              reject({ status: this.status,
                       statusText: "Parsing of " + url + " failed: " + err }
                   );
            }
          } else {
            resolve(d);
          }
        } else {
          reject({
            status: this.status,
            statusText: xhr.statusText
          });
        }
      };
      xhr.onerror = function () {
        reject({
          status: this.status,
          statusText: xhr.statusText
        });
      };
      xhr.send();
    });
  },

};
