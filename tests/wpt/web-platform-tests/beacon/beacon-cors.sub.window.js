// META: timeout=long
// META: script=/common/utils.js
// META: script=beacon-common.sub.js

"use strict";

// Execute each sample test with a cross-origin URL. If allowCors is 'true'
// the beacon handler will return CORS headers. This test ensures that the
// sendBeacon() succeeds in either case.
[true, false].forEach(function(allowCors) {
    // Implement the self.buildBaseUrl and self.buildTargetUrl extensions
    // to change the target URL to use a cross-origin domain name.
    self.buildBaseUrl = function(baseUrl) {
        return "http://{{domains[www]}}:{{ports[http][0]}}";
    };
    // Implement the self.buildTargetUrl extension to append a directive
    // to the handler, that it should return CORS headers, if 'allowCors'
    // is true.
    self.buildTargetUrl = function(targetUrl) {
        // Note that 'allowCors=true' is not necessary for the sendBeacon() to reach
        // the server. Beacons use the HTTP POST method, which is a CORS-safelisted
        // method, and thus they do not trigger preflight. If the server does not
        // respond with Access-Control-Allow-Origin and Access-Control-Allow-Credentials
        // headers, an error will be printed to the console, but the request will
        // already have reached the server. Since beacons are fire-and-forget, the
        // error will not affect any client script, either -- not even the return
        // value of the sendBeacon() call, because the underlying fetch is asynchronous.
        // The "Beacon CORS" tests are merely testing that sendBeacon() to a cross-
        // origin URL *will* work regardless.
        return allowCors ? `${targetUrl}&origin=http://{{host}}:{{ports[http][0]}}&credentials=true` : targetUrl;
    }

    const tests = [];
    for (const test of sampleTests) {
        const copy = Object.assign({}, test);
        copy.id = `${test.id}-${allowCors ? "CORS-ALLOW" : "CORS-FORBID"}`;
        tests.push(copy);
    }
    runTests(tests);
});

// Now test a cross-origin request that doesn't use a safelisted Content-Type and ensure
// we are applying the proper restrictions. Since a non-safelisted Content-Type request
// header is used there should be a preflight/options request and we should only succeed
// send the payload if the proper CORS headers are used.
{
    // Implement the self.buildBaseUrl and self.buildTargetUrl extensions
    // to change the target URL to use a cross-origin domain name.
    self.buildBaseUrl = function (baseUrl) {
        return "http://{{domains[www]}}:{{ports[http][0]}}";
    };

    // Implement the self.buildTargetUrl extension to append a directive
    // to the handler, that it should return CORS headers for the preflight we expect.
    self.buildTargetUrl = function (targetUrl) {
        return `${targetUrl}&origin=http://{{host}}:{{ports[http][0]}}&credentials=true&preflightExpected=true`;
    }
    const tests = [];
    for (const test of preflightTests) {
        const copy = Object.assign({}, test);
        copy.id = `${test.id}-PREFLIGHT-ALLOW`;
        tests.push(copy);
    }
    runTests(tests);
}

done();
