// META: timeout=long
// META: script=/common/utils.js
// META: script=beacon-common.sub.js

"use strict";

// Execute each sample test per redirect status code.
// Note that status codes 307 and 308 are the only codes that will maintain POST data
// through a redirect.
[307, 308].forEach(function(status) {
    function buildUrl(id) {
        const baseUrl = "http://{{host}}:{{ports[http][0]}}";
        const targetUrl = `${baseUrl}/beacon/resources/beacon.py?cmd=store&id=${id}`;

        return `/common/redirect.py?status=${status}&location=${encodeURIComponent(targetUrl)}`;
    }
    runTests(sampleTests, `-${status}`, buildUrl);
});

done();
