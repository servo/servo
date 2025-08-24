// META: script=helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

const frameSourceUrl =
    'https://{{hosts[alt][www]}}:{{ports[https][0]}}/storage-access-api/resources/sandboxed-iframe-allow-storage-access.html';

const sandboxAttribute =
    'allow-scripts allow-same-origin allow-storage-access-by-user-activation allow-popups';

RunTestsInIFrame(frameSourceUrl, sandboxAttribute);
