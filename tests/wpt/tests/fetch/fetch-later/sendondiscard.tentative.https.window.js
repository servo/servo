// META: script=/resources/testharness.js
// META: script=/resources/testharnessreport.js
// META: script=/common/utils.js
// META: script=/pending-beacon/resources/pending_beacon-helper.js

'use strict';

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);
  const numPerMethod = 20;
  const total = numPerMethod * 2;

  // Loads an iframe that creates `numPerMethod` GET & POST fetchLater requests.
  const iframe = await loadScriptAsIframe(`
    const url = "${url}";
    for (let i = 0; i < ${numPerMethod}; i++) {
      let get = fetchLater(url);
      let post = fetchLater(url, {method: 'POST'});
    }
  `);

  // Delete the iframe to trigger deferred request sending.
  document.body.removeChild(iframe);

  // The iframe should have sent all requests.
  await expectBeacon(uuid, {count: total});
}, 'A discarded document sends all its fetchLater requests with default config.');
