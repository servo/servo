// META: script=/common/utils.js
// META: script=/pending-beacon/resources/pending_beacon-helper.js
// META: timeout=long

'use strict';

parallelPromiseTest(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid);
  const numPerMethod = 20;
  const total = numPerMethod * 2;

  // Loads an iframe that creates `numPerMethod` GET & POST fetchLater requests.
  const iframe = await loadScriptAsIframe(`
    const url = '${url}';
    for (let i = 0; i < ${numPerMethod}; i++) {
      // Changing the URL of each request to avoid HTTP Cache issue.
      // See crbug.com/1498203#c17.
      fetchLater(url + "&method=GET&i=" + i,
        {method: 'GET', activateAfter: 10000});   // 10s
      fetchLater(url + "&method=POST&i=" + i,
        {method: 'POST', activateAfter: 8000});  // 8s
    }
  `);
  // Delete the iframe to trigger deferred request sending.
  document.body.removeChild(iframe);

  // The iframe should have sent all requests.
  await expectBeacon(uuid, {count: total});
}, 'A discarded document sends all its fetchLater requests, no matter how much their activateAfter timeout remain.');
