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
//   - self.buildBaseUrl: if present, can change the base URL of a beacon target URL (this
//     is the scheme, hostname, and port).
//   - self.buildTargetUrl: if present, can modify a beacon target URL (for example wrap it).
// Parameters:
//     testCases: An array of test cases.
//     sendData [optional]: A function that sends the beacon.
function runTests(testCases, sendData = self.sendData) {
    const session = initSession(testCases);

    testCases.forEach(function(testCase, testIndex) {
        // Make a copy of the test case as we'll be storing some metadata on it,
        // such as which session it belongs to.
        const testCaseCopy = Object.assign({ session: session }, testCase);

        testCaseCopy.index = testIndex;

        async_test((test) => {
            // Save the testharness.js 'test' object, so that we only have one object
            // to pass around.
            testCaseCopy.test = test;

            // Extension point: generate the beacon URL.
            var baseUrl = "http://{{host}}:{{ports[http][0]}}";
            if (self.buildBaseUrl) {
                baseUrl = self.buildBaseUrl(baseUrl);
            }
            var targetUrl = `${baseUrl}/beacon/resources/beacon.py?cmd=store&sid=${session.id}&tid=${testCaseCopy.id}&tidx=${testIndex}`;
            if (self.buildTargetUrl) {
                targetUrl = self.buildTargetUrl(targetUrl);
            }
            // Attach the URL to the test object for debugging purposes.
            testCaseCopy.url = targetUrl;

            assert_true(sendData(testCaseCopy), 'sendBeacon should succeed');
            waitForResult(testCaseCopy).then(() => test.done(), test.step_func((e) => {throw e;}));
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
    return self.navigator.sendBeacon(testCase.url, testCase.data);
}

// Poll the server for the test result.
async function waitForResult(testCase) {
    const session = testCase.session;
    const index = testCase.index;
    const url = `resources/beacon.py?cmd=stat&sid=${session.id}&tidx_min=${index}&tidx_max=${index}`;
    for (let i = 0; i < 30; ++i) {
        const response = await fetch(url);
        const text = await response.text();
        const results = JSON.parse(text);

        if (results.length === 0) {
          await new Promise(resolve => step_timeout(resolve, 100));
          continue;
        }
        assert_equals(results.length, 1, `bad response: '${text}'`);;
        // null JSON values parse as null, not undefined
        assert_equals(results[0].error, null, "'sendbeacon' data must not fail validation");
        return;
    }
    assert_true(false, 'timeout');
}

// Creates an iframe on the document's body and runs the sample tests from the iframe.
// The iframe is navigated immediately after it sends the data, and the window verifies
// that the data is still successfully sent.
function runSendInIframeAndNavigateTests() {
    var iframe = document.createElement("iframe");
    iframe.id = "iframe";
    iframe.onload = function() {
        // Clear our onload handler to prevent re-running the tests as we navigate away.
        iframe.onload = null;
        function sendData(testCase) {
            return iframe.contentWindow.navigator.sendBeacon(testCase.url, testCase.data);
        }
        const tests = [];
        for (const test of sampleTests) {
            const copy = Object.assign({}, test);
            copy.id = `${test.id}-NAVIGATE`;
            tests.push(copy);
        }
        runTests(tests, sendData);
        // Now navigate ourselves.
        iframe.contentWindow.location = "http://{{host}}:{{ports[http][0]}}/";
    };

    iframe.srcdoc = '<html></html>';
    document.body.appendChild(iframe);
}
