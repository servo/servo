// META: timeout=long
// META: script=/common/utils.js
// META: script=beacon-common.sub.js

"use strict";

// Execute each sample test with a cross-origin URL. If allowCors is 'true'
// the beacon handler will return CORS headers. This test ensures that the
// sendBeacon() succeeds in either case.
[true, false].forEach(function(allowCors) {
    function buildUrl(id) {
        const baseUrl = "http://{{domains[www]}}:{{ports[http][0]}}";
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
        const additionalQuery = allowCors ? "&origin=http://{{host}}:{{ports[http][0]}}&credentials=true" : "";
        return `${baseUrl}/beacon/resources/beacon.py?cmd=store&id=${id}${additionalQuery}`
    }
    runTests(sampleTests, allowCors ? "-CORS-ALLOW" : "-CORS-FORBID", buildUrl);
});

// Now test a cross-origin request that doesn't use a safelisted Content-Type and ensure
// we are applying the proper restrictions. Since a non-safelisted Content-Type request
// header is used there should be a preflight/options request and we should only succeed
// send the payload if the proper CORS headers are used.
{
    function buildUrl(id) {
        const baseUrl = "http://{{domains[www]}}:{{ports[http][0]}}";
        const additionalQuery = "&origin=http://{{host}}:{{ports[http][0]}}&credentials=true&preflightExpected=true";
        return `${baseUrl}/beacon/resources/beacon.py?cmd=store&id=${id}${additionalQuery}`
    }
    runTests(preflightTests, "-PREFLIGHT-ALLOW", buildUrl);
}

done();
