/* globals Promise, window, done, assert_true, on_event, promise_test */

/**
 * Creates an ATTAcomm object.  If the parameters are supplied
 * it sets up event listeners to send the test data to an ATTA if one
 * is available.  If the ATTA does not respond, it will assume the test
 * is being done manually and the results are being entered in the
 * parent test window.
 *
 * @constructor
 * @param {object} params
 * @param {string} [params.test] - object containing JSON test definition
 * @param {string} [params.testFile] - URI of a file with JSON test definition
 * @param {string} params.ATTAuri - URI to use to exercise the window
 * @event DOMContentLoaded Calls go once DOM is fully loaded
 * @returns {object} Reference to the new object
 *
 */

function ATTAcomm(params) {
  'use strict';

  this.Params = null;       // parameters passed in
  this.Promise = null;      // master Promise that resolves when intialization is complete
  this.Properties = null;   // testharness_properties from the opening window
  this.Tests = null;        // test object being processed
  this.testName = "";       // name of test being run
  this.log = "";            // a buffer to capture log information for debugging
  this.startReponse = {};   // startTest response will go in here for debugging

  this.loading = true;

  this.timeout = 5000;

  var pending = [] ;

  // set up in case DOM finishes loading early
  pending.push(new Promise(function(resolve) {
    on_event(document, "DOMContentLoaded", function() {
        resolve(true);
    }.bind(this));
  }.bind(this)));

  // if we are under runner, then there are props in the parent window
  //
  // if "output" is set in that, then pause at the end of running so the output
  // can be analyzed. @@@TODO@@@
  if (window && window.opener && window.opener.testharness_properties) {
    this.Properties = window.opener.testharness_properties;
  }

  this.Params = params;

  if (this.Params.hasOwnProperty("ATTAuri")) {
    this.ATTAuri = this.Params.ATTAuri;
  } else {
    this.ATTAuri = "http://localhost:4119";
  }

  if (this.Params.hasOwnProperty("title")) {
    this.testName = this.Params.title;
  }

  // start by loading the test (it might be inline, but
  // loadTest deals with that
  pending.push(this.loadTest(params)
    .then(function(tests) {
      // if the test is NOT an object, turn it into one
      if (typeof tests === 'string') {
        tests = JSON.parse(tests) ;
      }

      this.Tests = tests;

    }.bind(this)));

  this.Promise = new Promise(function(resolve, reject) {
    // once the DOM and the test is loaded... set us up
    Promise.all(pending)
    .then(function() {
      // Everything is loaded
      this.loading = false ;
      // run the automated tests (or setup for manual testing)
      this.go();
      resolve(this);
    }.bind(this))
    .catch(function(err) {
      // loading the components failed somehow - report the errors and mark the test failed
      test( function() {
        assert_true(false, "Loading of test components failed: " +JSON.stringify(err)) ;
      }, "Loading test components");
      this.dumpLog();
      done() ;
      reject("Loading of test components failed: "+JSON.stringify(err));
      return ;
    }.bind(this));
  }.bind(this));

  return this;
}

ATTAcomm.prototype = {

  /**
   * go sets up the connection to the ATTA
   *
   * If that succeeds and the tests in this test file have methods for
   * the API supported by the ATTA, then it automatically runs those tests.
   *
   * Otherwise it sets up for manualt testing.
   */
  go: function() {
    'use strict';
    // everything is ready.  Let's talk to the ATTA
    this.startTest().then(function(res) {

      // start was successful - iterate over steps
      var API = res.body.API;

      var subtestsForAPI = false;

      // check main and potentially nested lists of tests for
      // tests with this API.  If any step is missing this API
      // mapping, then we need to be manual
      this.Tests.forEach(function(subtest) {
        if (subtest.hasOwnProperty("test") &&
            subtest.test.hasOwnProperty(API)) {
          // there is at least one subtest for this API so
          // this is a test that needs to be looked at by an atta
          subtestsForAPI = true;
        } else if (Array.isArray(subtest)) {
          subtest.forEach(function(st) {
            if (st.hasOwnProperty("test") &&
                st.test.hasOwnProperty(API)) {
              subtestsForAPI = true;
            }
          });
        }
      });

      if (subtestsForAPI) {
        this.runTests(API, this.Tests)
        .then(function() {
          // the tests all ran; close it out
          this.endTest().then(function() {
            this.dumpLog();
            done();
          }.bind(this));
        }.bind(this))
        .catch(function(err) {
          this.endTest().then(function() {
            this.dumpLog();
            done();
          }.bind(this));
        }.bind(this));
      } else {
        // we don't know this API for this test
        // but we ARE talking to an ATTA; skip this test
        this.dumpLog();
        if (window.opener && window.opener.completion_callback) {
          window.opener.completion_callback([], { status: 3, message: "No steps for AT API " + API } );
        } else {
          done();
        }
        // this.setupManualTest("Unknown AT API: " + API);
      }
    }.bind(this))
    .catch(function(res) {
      // startTest failed so just sit and wait for a manual test to occur
      if (res.timeout || res.status === 102) {
        this.setupManualTest("No response from ATTA at " + this.ATTAuri);
      } else if (res.status === 200 ) {
        this.setupManualTest(res.message);
      } else if (res.statusText === "No response from ATTA") {
        this.setupManualTest("");
      } else {
        this.setupManualTest("Error from ATTA: " + res.status + ": " + res.statusText);
      }
    }.bind(this));
  },

  runTests: function(API, collection) {
    // this method returns a promise

    return new Promise(function(resolve, reject) {
      // accumulate promises; complete when done
      var pending = [];
      var testCount = 0;

      this.sendEvents(API, collection)
      .then(function(eventStatus) {

        /* Loop strategy...
         *
         * If the the step is a 'test' then push it into the pending queue as a promise
         *
         * If the step is anything else, then if there is anything in pending, wait on it
         * Once it resolves, clear the queue and then execute the other step.
         *
         */
        collection.forEach(function(subtest) {
          //  what "type" of step in the sequence is this?
          var theType = "test" ;
          if (Array.isArray(subtest)) {
            // it is a group
            Promise.all(pending).then(function() {
              pending = [];
              // recursively run the tests
              pending.push(this.runTests(API, subtest));
            }.bind(this));
          } else if (subtest.hasOwnProperty("type")) {
            theType = subtest.type;
          }
          testCount++;
          if (theType === "test") {
            // this is a set of assertions that should be evaluated
            pending.push(this.runTest(testCount, API, subtest));
          } else if (theType === "script") {
            Promise.all(pending).then(function() {
              pending = [];
              // execute the script
              this.runScript(testCount, subtest);
            }.bind(this));
          } else if (theType === "attribute") {
            Promise.all(pending).then(function() {
              pending = [];
              // raise the event
              this.handleAttribute(testCount, subtest);
            }.bind(this));
          // } else {
          } else if (theType === "event") {
            Promise.all(pending).then(function() {
              pending = [];
              // raise the event
              this.raiseEvent(testCount, subtest);
            }.bind(this));
          // } else {
          }
        }.bind(this));

        Promise.all(pending)
        .then(function() {
          // this collection all ran
          if (eventStatus !== "NOEVENTS") {
            // there were some events at the beginning
            this.sendStopListen().then(function() {
              resolve(true);
            });
          } else {
            resolve(true);
          }
        }.bind(this));
      }.bind(this));
    }.bind(this));
  },

  setupManualTest: function(message) {
    // if we determine the test should run manually, then expose all of the conditions that are
    // in the TEST data structure so that a human can to the inspection and calculate the result
    //
    'use strict';

    var ref = document.getElementById("manualMode");
    if (ref) {
      // we have a manualMode block.  Populate it
      var content = "<h2>Manual Mode Enabled</h2><p>"+message+"</p>";
      if (this.Tests.hasOwnProperty("description")) {
        content += "<p>" + this.Tests.description + "</p>";
      }
      var theTable = "<table id='steps'><tr><th>Step</th><th>Type</th><th>Element ID</th><th>Assertions</th></tr>";
      this.Tests.forEach(function(subtest) {
        var type = "test";
        if (subtest.hasOwnProperty("type")) {
          type = subtest.type;
        }
        var id = "" ;
        if (subtest.hasOwnProperty("element")) {
          id = subtest.element;
        }
        theTable += "<tr><td class='step'>" + subtest.title +"</td>";
        theTable += "<td class='type'>" + type + "</td>";
        theTable += "<td class='element'>" + id +"</td>";

        // now what do we put over here? depends on the type
        if (type === "test") {
          // it is a test; dump the assertions
          theTable += "<td>" + this.buildAssertionTable(subtest.test) + "</td>";
        } else if (type === "attribute" ) {
          if (subtest.hasOwnProperty("attribute") && subtest.hasOwnProperty("value") && subtest.hasOwnProperty("element")) {
            if (subtest.value === "none") {
              theTable += "<td>Remove attribute <code>" + subtest.attribute + "</code> from the element with ID <code>" + subtest.element + "</code></td>";
            } else {
              theTable += "<td>Set attribute <code>" + subtest.attribute + "</code> on the element with ID <code>" + subtest.element + "</code> to the value <code>" + subtest.value + "</code></td>";
            }
          }
        } else if (type === "event" ) {
          // it is some events
          if (subtest.hasOwnProperty("event") && subtest.hasOwnProperty("element")) {
            theTable += "<td>Send event <code>" + subtest.event + "</code> to the element with ID <code>" + subtest.element + "</code></td>";
          }
        } else if (type === "script" ) {
          // it is a script fragment
          theTable += "<td>Script: " + subtest.script + "</td>";
        } else {
          theTable += "<td>Unknown type: " + type + "</td>";
        }
        theTable += "</tr>";


      }.bind(this));

      theTable += "</table>";
      ref.innerHTML = content + theTable ;
    }
  },

  buildAssertionTable:  function(asserts) {
    "use strict";
    var output = "<table class='api'><tr><th>API Name</th><th colspan='4'>Assertions</th></tr>";
    var APIs = [] ;
    for (var k in asserts) {
      if (asserts.hasOwnProperty(k)) {
        APIs.push(k);
      }
    }

    APIs.sort().forEach(function(theAPI) {
      var rows = asserts[theAPI] ;
      var height = rows.length;
      output += "<tr><td rowspan='" + height + "' class='apiName'>"+theAPI+"</td>";
      var lastRow = rows.length - 1;
      rows.forEach(function(theRow, index) {
        var span = 4 - theRow.length;
        var colspan = span ? " colspan='"+span+"'" : "";
        theRow.forEach(function(item) {
          output += "<td" + colspan + ">" + item + "</td>";
        });
        output += "</tr>";
        if (index < lastRow) {
          output += "<tr>";
        }
      });
    });

    output += "</table>";
    return output;
  },

  // eventList - find the events for an API
  //
  // @param {string} API
  // @param {array} collection - a collection of tests
  // @returns {array} list of event names

  eventList: function(API, collection) {
    var eventHash = {};

    if (!API || API === "") {
      return [];
    }

    collection.forEach(function(subtest) {
      if (subtest.hasOwnProperty("test") &&
          subtest.test.hasOwnProperty(API)) {
        // this is a subtest for this API; look at the events
        subtest.test[API].forEach(function(assert) {
          // look for event names
          if (assert[0] === "event" && assert[1] === "type" && assert[2] === "is") {
            eventHash[assert[3]] = 1;
          }
        });
      }
    });

    return Object.keys(eventHash);
  },

  // handleAttribute - set or clear an attribute
  /**
   * @param {integer} testNum - The subtest number
   * @param {object} subtest - attribute information to set
   */
  handleAttribute: function(testNum, subtest) {
    "use strict";
    if (subtest) {
      if (subtest.hasOwnProperty("attribute") && subtest.hasOwnProperty("element") && subtest.hasOwnProperty("value")) {
        // update an attribute
        try {
          var node = document.getElementById(subtest.element);
          if (node) {
            if (subtest.value === "none") {
              // remove this attribute
              node.removeAttribute(subtest.attribute);
            } else if (subtest.value === '""') {
              node.setAttribute(subtest.attribute, "");
            } else if (subtest.value.match(/^"/) ) {
              var v = subtest.value;
              v = v.replace(/^"/, '');
              v = v.replace(/"$/, '');
              node.setAttribute(subtest.attribute, v);
            } else {
              node.setAttribute(subtest.attribute, subtest.value);
            }
          }
        }
        catch (e) {
          test(function() {
            assert_true(false, "Subtest attribute failed to update: " +e);
          }, "Attribute subtest " + testNum);
        }
      } else {
        test(function() {
          var err = "";
          if (!subtest.hasOwnProperty("attribute")) {
            err += "Attribute subtest has no attribute property; ";
          } else if (!subtest.hasOwnProperty("value")) {
            err += "Attribute subtest has no value property; ";
          } else if (!subtest.hasOwnProperty("element")) {
            err += "Attribute subtest has no element property; ";
          }
          assert_true(false, err);
        }, "Attribute subtest " + testNum );
      }
    }
    return;
  },



  // raiseEvent - throw an event at an item
  /**
   * @param {integer} testNum - The subtest number
   * @param {object} subtest - event information to throw
   */
  raiseEvent: function(testNum, subtest) {
    "use strict";
    var evt;
    if (subtest) {
      var kp = function(target, key) {
        evt = document.createEvent("KeyboardEvent");
        evt.initKeyEvent ("keypress", true, true, window,
                          0, 0, 0, 0, 0, "e".charCodeAt(0));
        target.dispatchEvent(evt);
      };
      if (subtest.hasOwnProperty("event") && subtest.hasOwnProperty("element")) {
        // throw an event
        try {
          var node = document.getElementById(subtest.element);
          if (node) {
            if (subtest.event === "focus") {
              node.focus();
            } else if (subtest.event === "select") {
              node.click();
            } else if (subtest.event.startsWith('key:')) {
              var key = subtest.event.replace('key:', '');
              evt = new KeyboardEvent("keypress", { "key": key});
              node.dispatchEvent(evt);
            } else {
              evt = new Event(subtest.element);
              node.dispatchEvent(evt);
            }
          }
        }
        catch (e) {
          test(function() {
            assert_true(false, "Subtest event failed to dispatch: " +e);
          }, "Event subtest " + testNum);
        }
      } else {
        test(function() {
          var err = "";
          if (!subtest.hasOwnProperty("event")) {
            err += "Event subtest has no event property; ";
          } else if (!subtest.hasOwnProperty("element")) {
            err += "Event subtest has no element property; ";
          }
          assert_true(false, err);
        }, "Event subtest " + testNum );
      }
    }
    return;
  },

  // runScript - run a script in the context of the window
  /**
   * @param {integer} testNum - The subtest number
   * @param {object} subtest - script and related information
   */
  runScript: function(testNum, subtest) {
    "use strict";
    if (subtest) {
      if (subtest.hasOwnProperty("script") && typeof subtest.script === "string") {
        try {
          /* jshint evil:true */
          eval(subtest.script);
        }
        catch (e) {
          test(function() {
            assert_true(false, "Subtest script " + subtest.script + " failed to evaluate: " +e);
          }, "Event subtest " + testNum);
        }
      } else {
        test(function() {
          assert_true(false, "Event subtest has no script property");
        }, "Event subtest " + testNum );
      }
    }
    return;
  },

  // runTest - process subtest
  /**
   * @param {integer} testNum - The subtest number
   * @param {string} API - name of the API being tested
   * @param {object} subtest - a subtest to run; contains 'title', 'element', and
   * 'test array'
   * @returns {Promise} - a Promise that resolves when the test completes
   */
  runTest: function(testNum, API, subtest) {
    'use strict';

    var data = {
      "title" : subtest.title,
      "id" : subtest.element,
      "data": this.normalize(subtest.test[API])
    };

    return new Promise(function(resolve) {
      var ANNO = this;
      if (subtest.test[API]) {
        // we actually have a test to run
        promise_test(function() {
          // force a resolve of the promise regardless
          this.add_cleanup(function() { resolve(true); });
          return ANNO.sendTest(data)
            .then(function(res) {
              if (typeof res.body === "object" && res.body.hasOwnProperty("status")) {
                // we got some sort of response
                if (res.body.status === "OK") {
                  // the test ran - yay!
                  var messages = "";
                  var thisResult = null;
                  var theLog = "";
                  var assertionCount = 0;
                  res.body.results.forEach( function (a) {
                    if (typeof a === "object") {
                      // we have a result for this assertion
                      // first, what is the assertion?
                      var aRef = data.data[assertionCount];
                      var assertionText = '"' + aRef.join(" ") +'"';

                      if (a.hasOwnProperty("log") && a.log !== null && a.log !== '' ) {
                        // there is log data - save it
                        theLog += "\n--- Assertion " + assertionCount + " ---";
                        theLog += "\nAssertion: " + assertionText + "\nLog data: "+a.log ;
                      }

                      // is there a message?
                      var theMessage = "";
                      if (a.hasOwnProperty("message")) {
                        theMessage = a.message;
                      }
                      if (!a.hasOwnProperty("result")) {
                        messages += "ATTA did not report a result " + theMessage + "; ";
                      } else if (a.result === "ERROR") {
                        messages += "ATTA reported ERROR with message: " + theMessage + "; ";
                      } else if (a.result === "FAIL") {
                        thisResult = false;
                        messages += assertionText + " failed " + theMessage + "; ";
                      } else if (a.result === "PASS" && thisResult === null) {
                        // if we got a pass and there was no other result thus far
                        // then we are passing
                        thisResult = true;
                      }
                    }
                    assertionCount++;
                  });
                  if (theLog !== "") {
                    ANNO.saveLog("runTest", theLog, subtest);
                  }
                  if (thisResult !== null) {
                    assert_true(thisResult, messages);
                  } else {
                    assert_true(false, "ERROR: No results reported from ATTA; " + messages);
                  }
                } else if (res.body.status === "ERROR") {
                  assert_true(false, "ATTA returned ERROR with message: " + res.body.statusText);
                } else {
                  assert_true(false, "ATTA returned unknown status " + res.body.status + " with message: " + res.body.statusText);
                }
              } else {
                // the return wasn't an object!
                assert_true(false, "ATTA failed to return a result object: returned: "+JSON.stringify(res));
              }
            });
        }, subtest.name );
      } else {
        // there are no test steps for this API.  fake a subtest result
        promise_test(function() {
          // force a resolve of the promise regardless
          this.add_cleanup(function() { resolve(true); });
          return new Promise(function(innerResolve) {
            innerResolve(true);
          })
          .then(function(res) {
            var theLog = "\nSUBTEST NOTRUN: No assertions for API " + API + "\n";
            if (theLog !== "") {
              ANNO.saveLog("runTest", theLog, subtest);
            }
            assert_false(true, "NOTRUN: No assertion for API " + API);
          });
        }, subtest.name );
      }
    }.bind(this));
  },

  // loadTest - load a test from an external JSON file
  //
  // returns a promise that resolves with the contents of the
  // test

  loadTest: function(params) {
    'use strict';

    if (params.hasOwnProperty('stepFile')) {
      // the test is referred to by a file name
      return this._fetch("GET", params.stepFile);
    } // else
    return new Promise(function(resolve, reject) {
      if (params.hasOwnProperty('steps')) {
        resolve(params.steps);
      } else {
        reject("Must supply a 'steps' or 'stepFile' parameter");
      }
    });
  },

  /* dumpLog - put log information into the log div on the page if it exists
   */

  dumpLog: function() {
    'use strict';
    if (this.log !== "") {
      var ref = document.getElementById("ATTAmessages");
      if (ref) {
        // we have a manualMode block.  Populate it
        var content = "<h2>Logging information recorded</h2>";
        if (this.startResponse && this.startResponse.hasOwnProperty("API")) {
          content += "<h3>ATTA Information</h3>";
          content += "<pre>"+JSON.stringify(this.startResponse, null, "  ")+"</pre>";
        }
        content += "<textarea rows='50' style='width:100%'>"+this.log+"</textarea>";
        ref.innerHTML = content ;
      }
    }
  },

  /* saveLog - capture logging information so that it can be displayed on the page after testing is complete
   *
   * @param {string} caller name
   * @param {string} log message
   * @param {object} subtest
   */

  saveLog: function(caller, message, subtest) {
    'use strict';

    if (typeof message === "string" && message !== "") {
      this.log += "============================================================\n";
      this.log += "Message from " + caller + "\n";
      if (subtest && typeof subtest === "object") {
        var API = this.startResponse.API;
        this.log += "\n    SUBTEST TITLE: " + subtest.title;
        this.log += "\n  SUBTEST ELEMENT: " + subtest.element;
        this.log += "\n     SUBTEST DATA: " + JSON.stringify(subtest.test[API]);
        this.log += "\n\n";
      }
      this.log += message;
    }
    return;
  },

  // startTest - send the test start message
  //
  // @returns {Promise} resolves if the start is successful, or rejects with

  startTest: function() {
    'use strict';

    return new Promise(function(resolve, reject) {
      var params = {
        test: this.testName || window.title,
        url: document.location.href
      };

      this._fetch("POST", this.ATTAuri + "/start", null, params)
      .then(function(res) {
        if (res.body.hasOwnProperty("status")) {
          if (res.body.status === "READY") {
            this.startResponse = res.body;
            if (res.body.hasOwnProperty("log")) {
              // there is some logging data - capture it
              this.saveLog("startTest", res.body.log);
            }
            // the system is ready for us - is it really?
            if (res.body.hasOwnProperty("API")) {
              resolve(res);
            } else {
              res.message = "No API in response from ATTA";
              reject(res);
            }
          } else {
            // the system reported something else - fail out with the statusText as a result
            res.message = "ATTA reported an error: " + res.body.statusText;
            reject(res);
          }
        } else {
          res.message = "ATTA did not report a status";
          reject(res);
        }
      }.bind(this))
      .catch(function(res) {
        reject(res);
      });
    }.bind(this));
  },

  // sendEvents - send the list of events the ATTA needs to listen for
  //
  // @param {string} API
  // @param {array} collection - a list of tests
  // @returns {Promise} resolves if the message is successful, or rejects with

  sendEvents: function(API, collection) {
    'use strict';

    return new Promise(function(resolve, reject) {
      var eList = this.eventList(API, collection) ;
      if (eList && eList.length) {
        var params = {
          events: eList
        };

        this._fetch("POST", this.ATTAuri + "/startlisten", null, params)
        .then(function(res) {
          if (res.body.hasOwnProperty("status")) {
            if (res.body.status === "READY") {
              if (res.body.hasOwnProperty("log")) {
                // there is some logging data - capture it
                this.saveLog("sendEvents", res.body.log);
              }
              resolve(res.body.status);
            } else {
              // the system reported something else - fail out with the statusText as a result
              res.message = "ATTA reported an error: " + res.body.statusText;
              reject(res);
            }
          } else {
            res.message = "ATTA did not report a status";
            reject(res);
          }
        }.bind(this))
        .catch(function(res) {
          reject(res);
        });
      } else {
        // there are no events
        resolve("NOEVENTS");
      }
    }.bind(this));
  },

  sendStopListen: function() {
    'use strict';

    return this._fetch("POST", this.ATTAuri + "/stoplisten", null, null);
  },

  // sendTest - send test data to an ATTA and wait for a response
  //
  // returns a promise that resolves with the results of the test

  sendTest: function(testData) {
    'use strict';

    if (typeof testData !== "string") {
      testData = JSON.stringify(testData);
    }
    var ret = this._fetch("POST", this.ATTAuri + "/test", null, testData, true);
    ret.then(function(res) {
      if (res.body.hasOwnProperty("log")) {
        // there is some logging data - capture it
        this.saveLog("sendTest", res.body.log);
      }
    }.bind(this));
    return ret;
  },

  endTest: function() {
    'use strict';

    return this._fetch("GET", this.ATTAuri + "/end");
  },

  /* normalize - ensure subtest data conforms to ATTA spec
   */

  normalize: function( data ) {
    'use strict';

    var ret = [] ;

    if (data) {
      data.forEach(function(assert) {
        var normal = [] ;
        // ensure if there is a value list it is compressed
        if (Array.isArray(assert)) {
          // we have an array
          normal[0] = assert[0];
          normal[1] = assert[1];
          normal[2] = assert[2];
          if ("string" === typeof assert[3] && assert[3].match(/^\[.*\]$/)) {
            // it is a string and matches the valuelist pattern
            normal[3] = assert[3].replace(/, +/, ',');
          } else {
            normal[3] = assert[3];
          }
          ret.push(normal);
        } else {
          ret.push(assert);
        }
      });
    }
    return ret;
  },

  // _fetch - return a promise after sending data
  //
  // Resolves with the returned information in a structure
  // including:
  //
  // xhr - a raw xhr object
  // headers - an array of headers sent in the request
  // status - the status code
  // statusText - the text of the return status
  // text - raw returned data
  // body - an object parsed from the returned content
  //

  _fetch: function (method, url, headers, content, parse) {
    'use strict';
    if (method === null || method === undefined) {
      method = "GET";
    }
    if (parse === null || parse === undefined) {
      parse = true;
    }
    if (headers === null || headers === undefined) {
      headers = [];
    }


    // note that this Promise always resolves - there is no reject
    // condition

    return new Promise(function (resolve, reject) {
      var xhr = new XMLHttpRequest();

      // this gets returned when the request completes
      var resp = {
        xhr: xhr,
        headers: null,
        status: 0,
        statusText: "",
        body: null,
        text: ""
      };

      xhr.open(method, url);

      // headers?
      headers.forEach(function(ref) {
        xhr.setRequestHeader(ref[0], ref[1]);
      });

      //if (this.timeout) {
      //  xhr.timeout = this.timeout;
      //}

      xhr.ontimeout = function() {
        resp.timeout = this.timeout;
        resolve(resp);
      };

      xhr.onerror = function() {
        if (this.status) {
          resp.status = this.status;
          resp.statusText = xhr.statusText;
        } else if (this.status === 0) {
          resp.status = 0;
          resp.statusText = "No response from ATTA";
        }
        reject(resp);
      };

      xhr.onload = function () {
        resp.status = this.status;
        if (this.status >= 200 && this.status < 300) {
          var d = xhr.response;
          // return the raw text of the response
          resp.text = d;
          // we have it; what is it?
          if (parse) {
            try {
              d = JSON.parse(d);
              resp.body = d;
            }
            catch(err) {
              resp.body = null;
            }
          }
          resolve(resp);
        } else {
          reject({
            status: this.status,
            statusText: xhr.statusText
          });
        }
      };

      if (content !== null && content !== undefined) {
        if ("object" === typeof(content)) {
          xhr.send(JSON.stringify(content));
        } else if ("function" === typeof(content)) {
          xhr.send(content());
        } else if ("string" === typeof(content)) {
          xhr.send(content);
        }
      } else {
        xhr.send();
      }
    });
  },

};

// vim: set ts=2 sw=2:
