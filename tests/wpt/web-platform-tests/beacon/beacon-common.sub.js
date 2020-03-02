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
//      name: String containing the unique name of the test case.
//      data: Payload object to send through sendbeacon.
var noDataTest = { name: "NoData" };
var nullDataTest = { name: "NullData", data: null };
var undefinedDataTest = { name: "UndefinedData", data: undefined };
var smallStringTest = { name: "SmallString", data: smallPayload };
var mediumStringTest = { name: "MediumString", data: mediumPayload };
var largeStringTest = { name: "LargeString", data: largePayload };
var maxStringTest = { name: "MaxString", data: maxPayload };
var emptyBlobTest = { name: "EmptyBlob", data: new Blob() };
var smallBlobTest = { name: "SmallBlob", data: new Blob([smallPayload]) };
var mediumBlobTest = { name: "MediumBlob", data: new Blob([mediumPayload]) };
var largeBlobTest = { name: "LargeBlob", data: new Blob([largePayload]) };
var maxBlobTest = { name: "MaxBlob", data: new Blob([maxPayload]) };
var emptyBufferSourceTest = { name: "EmptyBufferSource", data: new Uint8Array() };
var smallBufferSourceTest = { name: "SmallBufferSource", data: CreateArrayBufferFromPayload(smallPayload) };
var mediumBufferSourceTest = { name: "MediumBufferSource", data: CreateArrayBufferFromPayload(mediumPayload) };
var largeBufferSourceTest = { name: "LargeBufferSource", data: CreateArrayBufferFromPayload(largePayload) };
var maxBufferSourceTest = { name: "MaxBufferSource", data: CreateArrayBufferFromPayload(maxPayload) };
var emptyFormDataTest = { name: "EmptyFormData", data: CreateEmptyFormDataPayload() };
var smallFormDataTest = { name: "SmallFormData", data: CreateFormDataFromPayload(smallPayload) };
var mediumFormDataTest = { name: "MediumFormData", data: CreateFormDataFromPayload(mediumPayload) };
var largeFormDataTest = { name: "LargeFormData", data: CreateFormDataFromPayload(largePayload) };
var smallSafeContentTypeEncodedTest = { name: "SmallSafeContentTypeEncoded", data: new Blob([smallPayload], { type: 'application/x-www-form-urlencoded' }) };
var smallSafeContentTypeFormTest = { name: "SmallSafeContentTypeForm", data: new FormData() };
var smallSafeContentTypeTextTest = { name: "SmallSafeContentTypeText", data: new Blob([smallPayload], { type: 'text/plain' }) };
var smallCORSContentTypeTextTest = { name: "SmallCORSContentTypeText", data: new Blob([smallPayload], { type: 'text/html' }) };
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

// Schedules async_test's for each of the test cases, treating them as a single session,
// and wires up the continueAfterSendingBeacon() and waitForResults() calls.
// Parameters:
//     testCases: An array of test cases.
//     suffix [optional]: A string used for the suffix for each test case name.
//     buildUrl [optional]: A function that returns a beacon URL given an id.
//     sendData [optional]: A function that sends the beacon with given a URL and payload.
function runTests(testCases, suffix = '', buildUrl = self.buildUrl, sendData = self.sendData) {
    for (const testCase of testCases) {
        const id = token();
        async_test((test) => {
            const url = buildUrl(id);
            assert_true(sendData(url, testCase.data), 'sendBeacon should succeed');
            waitForResult(id).then(() => test.done(), test.step_func((e) => {throw e;}));
        }, `Verify 'navigator.sendbeacon()' successfully sends for variant: ${testCase.name}${suffix}`);
    };
}

function buildUrl(id) {
    const baseUrl = "http://{{host}}:{{ports[http][0]}}";
    return `${baseUrl}/beacon/resources/beacon.py?cmd=store&id=${id}`;
}

// Sends the beacon for a single test. This step is factored into its own function so that
// it can be called from a web worker. It does not check for results.
// Note: do not assert from this method, as when called from a worker, we won't have the
// full testharness.js test context. Instead return 'false', and the main scope will fail
// the test.
// Returns the result of the 'sendbeacon()' function call, true or false.
function sendData(url, payload) {
    return self.navigator.sendBeacon(url, payload);
}

// Poll the server for the test result.
async function waitForResult(id) {
    const url = `resources/beacon.py?cmd=stat&id=${id}`;
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
        function sendData(url, payload) {
            return iframe.contentWindow.navigator.sendBeacon(url, payload);
        }
        runTests(sampleTests, '-NAVIGATE', self.buildUrl, sendData);
        // Now navigate ourselves.
        iframe.contentWindow.location = "http://{{host}}:{{ports[http][0]}}/";
    };

    iframe.srcdoc = '<html></html>';
    document.body.appendChild(iframe);
}
