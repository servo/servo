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
      fetchLater(url);
      fetchLater(url, {method: 'POST'});
    }
  `);
  // Delete the iframe to trigger deferred request sending.
  document.body.removeChild(iframe);

  // The iframe should have sent all requests.
  await expectBeacon(uuid, {count: total});
}, 'A discarded document sends all its fetchLater requests.');

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);
  const numPerMethod = 20;
  const total = numPerMethod * 2;

  // Loads an iframe that creates `numPerMethod` GET & POST fetchLater requests.
  const iframe = await loadScriptAsIframe(`
    const url = "${url}";
    for (let i = 0; i < ${numPerMethod}; i++) {
      fetchLater(url, {method: 'GET', activationTimeout: 10000});  // 10s
      fetchLater(url, {method: 'POST', activationTimeout: 8000});  // 8s
    }
  `);
  // Delete the iframe to trigger deferred request sending.
  document.body.removeChild(iframe);

  // The iframe should have sent all requests.
  await expectBeacon(uuid, {count: total});
}, 'A discarded document sends all its fetchLater requests, no matter how much their timeout remain.');

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);

  // Loads an iframe that creates 2 fetchLater requests. One of them is aborted.
  const iframe = await loadScriptAsIframe(`
    const url = "${url}";
    const controller = new AbortController();
    fetchLater(url, {signal: controller.signal});
    fetchLater(url, {method: 'POST'});
    controller.abort();
  `);
  // Delete the iframe to trigger deferred request sending.
  document.body.removeChild(iframe);

  // The iframe should not send the aborted request.
  // TODO(crbug.com/1465781): Fix this after implementing abort function.
  await expectBeacon(uuid, {count: 1});
}, 'A discarded document does not send an already aborted fetchLater request.');
