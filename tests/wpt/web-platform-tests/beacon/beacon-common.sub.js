"use strict";

// Different sizes of payloads to test.
var smallPayloadSize = 10;
var mediumPayloadSize = 10000;
var largePayloadSize = 50000;
var maxPayloadSize = 65536; // The maximum payload size allowed for a beacon request.

// String payloads of various sizes sent by sendbeacon. The format of the payloads is a string:
//     <numberOfCharacters>:<numberOfCharacters *'s>
//     ex. "10:**********"
var smallPayload = smallPayloadSize + ":" + Array(smallPayloadSize).fill('*').join("");
var mediumPayload = mediumPayloadSize + ":" + Array(mediumPayloadSize).fill('*').join("");
var largePayload = largePayloadSize + ":" + Array(largePayloadSize).fill('*').join("");
// Subtract 6 from maxPayloadSize because 65536 is 5 digits, plus 1 more for the ':'
var maxPayload = (maxPayloadSize - 6) + ":" + Array(maxPayloadSize - 6).fill('*').join("")

// Test case definitions.
//      id: String containing the unique name of the test case.
//      data: Payload object to send through sendbeacon.
var noDataTest = { id: "NoData" };
var nullDataTest = { id: "NullData", data: null };
var undefinedDataTest = { id: "UndefinedData", data: undefined };
var smallStringTest = { id: "SmallString", data: smallPayload };
var mediumStringTest = { id: "MediumString", data: mediumPayload };
var largeStringTest = { id: "LargeString", data: largePayload };
var maxStringTest = { id: "MaxString", data: maxPayload };
var emptyBlobTest = { id: "EmptyBlob", data: new Blob() };
var smallBlobTest = { id: "SmallBlob", data: new Blob([smallPayload]) };
var mediumBlobTest = { id: "MediumBlob", data: new Blob([mediumPayload]) };
var largeBlobTest = { id: "LargeBlob", data: new Blob([largePayload]) };
var maxBlobTest = { id: "MaxBlob", data: new Blob([maxPayload]) };
var emptyBufferSourceTest = { id: "EmptyBufferSource", data: new Uint8Array() };
var smallBufferSourceTest = { id: "SmallBufferSource", data: CreateArrayBufferFromPayload(smallPayload) };
var mediumBufferSourceTest = { id: "MediumBufferSource", data: CreateArrayBufferFromPayload(mediumPayload) };
var largeBufferSourceTest = { id: "LargeBufferSource", data: CreateArrayBufferFromPayload(largePayload) };
var maxBufferSourceTest = { id: "MaxBufferSource", data: CreateArrayBufferFromPayload(maxPayload) };
var emptyFormDataTest = { id: "EmptyFormData", data: CreateEmptyFormDataPayload() };
var smallFormDataTest = { id: "SmallFormData", data: CreateFormDataFromPayload(smallPayload) };
var mediumFormDataTest = { id: "MediumFormData", data: CreateFormDataFromPayload(mediumPayload) };
var largeFormDataTest = { id: "LargeFormData", data: CreateFormDataFromPayload(largePayload) };
var smallSafeContentTypeEncodedTest = { id: "SmallSafeContentTypeEncoded", data: new Blob([smallPayload], { type: 'application/x-www-form-urlencoded' }) };
var smallSafeContentTypeFormTest = { id: "SmallSafeContentTypeForm", data: new FormData() };
var smallSafeContentTypeTextTest = { id: "SmallSafeContentTypeText", data: new Blob([smallPayload], { type: 'text/plain' }) };
var smallCORSContentTypeTextTest = { id: "SmallCORSContentTypeText", data: new Blob([smallPayload], { type: 'text/html' }) };
// We don't test maxFormData because the extra multipart separators make it difficult to
// calculate a maxPayload.

// Test case suites.
// Due to quota limits we split the max payload tests into their own bucket.
var stringTests = [noDataTest, nullDataTest, undefinedDataTest, smallStringTest, mediumStringTest, largeStringTest];
var stringMaxTest = [maxStringTest];
var blobTests = [emptyBlobTest, smallBlobTest, mediumBlobTest, largeBlobTest];
var blobMaxTest = [maxBlobTest];
var bufferSourceTests = [emptyBufferSourceTest, smallBufferSourceTest, mediumBufferSourceTest, largeBufferSourceTest];
var bufferSourceMaxTest = [maxBufferSourceTest];
var formDataTests = [emptyFormDataTest, smallFormDataTest, mediumFormDataTest, largeFormDataTest];
var formDataMaxTest = [largeFormDataTest];
var contentTypeTests = [smallSafeContentTypeEncodedTest,smallSafeContentTypeFormTest,smallSafeContentTypeTextTest,smallCORSContentTypeTextTest];
var allTests = [].concat(stringTests, stringMaxTest, blobTests, blobMaxTest, bufferSourceTests, bufferSourceMaxTest, formDataTests, formDataMaxTest, contentTypeTests);

// This special cross section of test cases is meant to provide a slimmer but reasonably-
// representative set of tests for parameterization across variables (e.g. redirect codes,
// cors modes, etc.)
var sampleTests = [noDataTest, nullDataTest, undefinedDataTest, smallStringTest, smallBlobTest, smallBufferSourceTest, smallFormDataTest, smallSafeContentTypeEncodedTest, smallSafeContentTypeFormTest, smallSafeContentTypeTextTest];

var preflightTests = [smallCORSContentTypeTextTest];

// Build a test lookup table, which is useful when instructing a web worker or an iframe
// to run a test, so that we don't have to marshal the entire test case across a process boundary.
var testLookup = {};
allTests.forEach(function(testCase) {
    testLookup[testCase.id] = testCase;
});

// Helper function to create an ArrayBuffer representation of a string.
function CreateArrayBufferFromPayload(payload) {
    var length = payload.length;
    var buffer = new Uint8Array(length);

    for (var i = 0; i < length; i++) {
        buffer[i] = payload.charCodeAt(i);
    }

    return buffer;
}

// Helper function to create an empty FormData object.
function CreateEmptyFormDataPayload() {
    if (self.document === undefined) {
        return null;
    }

    return new FormData();
}

// Helper function to create a FormData representation of a string.
function CreateFormDataFromPayload(payload) {
    if (self.document === undefined) {
        return null;
    }

    var formData = new FormData();
    formData.append("payload", payload);
    return formData;
}

// Initializes a session with a client-generated SID.
// A "session" is a run of one or more tests. It is used to batch several beacon
// tests in a way that isolates the server-side session state and makes it easy
// to poll the results of the tests in one request.
//     testCases: The array of test cases participating in the session.
function initSession(testCases) {
    return {
        // Provides a unique session identifier to prevent mixing server-side data
        // with other sessions.
        id: self.token(),
        // Dictionary of test name to live testCase object.
        testCaseLookup: {},
        // Array of testCase objects for iteration.
        testCases: [],
        // Tracks the total number of tests in the session.
        totalCount: testCases.length,
        // Tracks the number of tests for which we have sent the beacon.
        // When it reaches totalCount, we will start polling for results.
        sentCount: 0,
        // Tracks the number of tests for which we have verified the results.
        // When it reaches sentCount, we will stop polling for results.
        doneCount: 0,
        // Helper to add a testCase to the session.
        add: function add(testCase) {
            this.testCases.push(testCase);
            this.testCaseLookup[testCase.id] = testCase;
        }
    };
}

// Schedules async_test's for each of the test cases, treating them as a single session,
// and wires up the continueAfterSendingBeacon() and waitForResults() calls.
// The method looks for several "extension" functions in the global scope:
//   - self.buildId: if present, can change the display name of a test.
//   - self.buildBaseUrl: if present, can change the base URL of a beacon target URL (this
//     is the scheme, hostname, and port).
//   - self.buildTargetUrl: if present, can modify a beacon target URL (for example wrap it).
// Parameters:
//     testCases: An array of test cases.
function runTests(testCases) {
    var session = initSession(testCases);

    testCases.forEach(function(testCase, testIndex) {
        // Make a copy of the test case as we'll be storing some metadata on it,
        // such as which session it belongs to.
        var testCaseCopy = Object.assign({ session: session }, testCase);

        // Extension point: generate the test id.
        var testId = testCase.id;
        if (self.buildId) {
            testId = self.buildId(testId);
        }
        testCaseCopy.origId = testCaseCopy.id;
        testCaseCopy.id = testId;
        testCaseCopy.index = testIndex;

        session.add(testCaseCopy);

        // Schedule the sendbeacon in an async test.
        async_test(function(test) {
            // Save the testharness.js 'test' object, so that we only have one object
            // to pass around.
            testCaseCopy.test = test;

            // Extension point: generate the beacon URL.
            var baseUrl = "http://{{host}}:{{ports[http][0]}}";
            if (self.buildBaseUrl) {
                baseUrl = self.buildBaseUrl(baseUrl);
            }
            var targetUrl = `${baseUrl}/beacon/resources/beacon.py?cmd=store&sid=${session.id}&tid=${testId}&tidx=${testIndex}`;
            if (self.buildTargetUrl) {
                targetUrl = self.buildTargetUrl(targetUrl);
            }
            // Attach the URL to the test object for debugging purposes.
            testCaseCopy.url = targetUrl;

            // Extension point: send the beacon immediately, or defer.
            var sendFunc = test.step_func(function sendImmediately(testCase) {
                var sendResult = sendData(testCase);
                continueAfterSendingBeacon(sendResult, testCase);
            });
            if (self.sendFunc) {
                sendFunc = test.step_func(self.sendFunc);
            }
            sendFunc(testCaseCopy);
        }, `Verify 'navigator.sendbeacon()' successfully sends for variant: ${testCaseCopy.id}`);
    });
}

// Sends the beacon for a single test. This step is factored into its own function so that
// it can be called from a web worker. It does not check for results.
// Note: do not assert from this method, as when called from a worker, we won't have the
// full testharness.js test context. Instead return 'false', and the main scope will fail
// the test.
// Returns the result of the 'sendbeacon()' function call, true or false.
function sendData(testCase) {
    var sent = false;
    if (testCase.data) {
        sent = self.navigator.sendBeacon(testCase.url, testCase.data);
    } else {
        sent = self.navigator.sendBeacon(testCase.url)
    }
    return sent;
}

// Continues a single test after the beacon has been sent for that test.
// Will trigger waitForResults() for the session if this is the last test
// in the session to send its beacon.
// Assumption: will be called on the test's step_func so that assert's do
// not have to be wrapped.
function continueAfterSendingBeacon(sendResult, testCase) {
    var session = testCase.session;

    // Recaclulate the sent vs. total counts.
    if (sendResult) {
        session.sentCount++;
    } else {
        session.totalCount--;
    }

    // If this was the last test in the session to send its beacon, start polling for results.
    // Note that we start polling even if just one test in the session sends successfully,
    // so that if any of the others fail, we still get results from the tests that did send.
    if (session.sentCount == session.totalCount) {
        // Exit the current test's execution context in order to run the poll
        // loop from the harness context.
        step_timeout(waitForResults.bind(this, session), 0);
    }

    // Now fail this test if the beacon did not send. It will be excluded from the poll
    // loop because of the calculation adjustment above.
    assert_true(sendResult, "'sendbeacon' function call must succeed");
}

// Kicks off an asynchronous monitor to poll the server for test results. As we
// verify that the server has received and validated a beacon, we will complete
// its testharness test.
function waitForResults(session) {
    // Poll for status until all of the results come in.
    fetch(`resources/beacon.py?cmd=stat&sid=${session.id}&tidx_min=0&tidx_max=${session.totalCount-1}`).then(
        function(response) {
            // Parse as text(), not json(), so that we can log the raw response if
            // it's invalid.
            response.text().then(function(rawResponse) {
                // Check that we got a response we expect and know how to handle.
                var results;
                var failure;
                try {
                    results = JSON.parse(rawResponse);

                    if (results.length === undefined) {
                        failure = `bad validation response schema: rawResponse='${rawResponse}'`;
                    }
                } catch (e) {
                    failure = `bad validation response: rawResponse='${rawResponse}', got parse error '${e}'`;
                }

                if (failure) {
                    // At this point we can't deterministically get results for all of the
                    // tests in the session, so fail the entire session.
                    failSession(session, failure);
                    return;
                }

                // The 'stat' call will return an array of zero or more results
                // of sendbeacon() calls that the server has received and validated.
                results.forEach(function(result) {
                    var testCase = session.testCaseLookup[result.id];

                    // While stash.take on the server is supposed to honor read-once, since we're
                    // polling so frequently it is possible that we will receive the same test result
                    // more than once.
                    if (!testCase.done) {
                        testCase.done = true;
                        session.doneCount++;
                    }

                    // Validate that the sendbeacon() was actually sent to the server.
                    var test = testCase.test;
                    test.step(function() {
                        // null JSON values parse as null, not undefined
                        assert_equals(result.error, null, "'sendbeacon' data must not fail validation");
                    });

                    test.done();
                });

                // Continue polling until all of the results come in.
                if (session.doneCount < session.sentCount) {
                    // testharness.js frowns upon the use of explicit timeouts, but there is no way
                    // around the need to poll for these tests, and there is no use spamming the server
                    // with requestAnimationFrame() just to avoid the use of step_timeout.
                    step_timeout(waitForResults.bind(this, session), 100);
                }
            }).catch(function(error) {
                failSession(session, `unexpected error reading response, error='${error}'`);
            });
        }
    );
}

// Fails all of the tests in the session, meant to be called when an infrastructural
// issue prevents us from deterministically completing the individual tests.
function failSession(session, reason) {
    session.testCases.forEach(function(testCase) {
        var test = testCase.test;
        test.unreached_func(reason)();
    });
}

// Creates an iframe on the document's body and runs the sample tests from the iframe.
// The iframe is navigated immediately after it sends the data, and the window verifies
// that the data is still successfully sent.
//    funcName: "beacon" to send the data via navigator.sendBeacon(),
//              "fetch" to send the data via fetch() with the keepalive flag.
function runSendInIframeAndNavigateTests(funcName) {
    var iframe = document.createElement("iframe");
    iframe.id = "iframe";
    iframe.onload = function() {
        var tests = Array();

        // Clear our onload handler to prevent re-running the tests as we navigate away.
        this.onload = null;

        // Implement the self.buildId extension to identify the parameterized
        // test in the report.
        self.buildId = function(baseId) {
            return `${baseId}-${funcName}-NAVIGATE`;
        };

        window.onmessage = function(e) {
            // The iframe will execute sendData() for us and return the result.
            var testCase = tests[e.data];
            continueAfterSendingBeacon(true /* sendResult */, testCase);
        };

        // Implement the self.sendFunc extension to send the beacon indirectly,
        // from an iFrame that we can then navigate.
        self.sendFunc = function(testCase) {
            var iframeWindow = document.getElementById("iframe").contentWindow;
            // We run into problems passing the testCase over the document boundary,
            // because of structured cloning constraints. Instead we'll send over the
            // test case id, and the iFrame can load the static test case by including
            // beacon-common.js.
            tests[testCase.origId] = testCase;
            iframeWindow.postMessage([testCase.origId, testCase.url, funcName], "*");
        };

        runTests(sampleTests);
    };

    iframe.src = "navigate.iFrame.sub.html";
    document.body.appendChild(iframe);
}
