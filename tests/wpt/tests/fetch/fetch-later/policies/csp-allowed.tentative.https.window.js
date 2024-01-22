// META: title=FetchLater: allowed by CSP
// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js
// META: script=/pending-beacon/resources/pending_beacon-helper.js
'use strict';

const {
  HTTPS_NOTSAMESITE_ORIGIN,
} = get_host_info();

// FetchLater requests allowed by Content Security Policy.
// https://w3c.github.io/webappsec-csp/#should-block-request

const meta = document.createElement('meta');
meta.setAttribute('http-equiv', 'Content-Security-Policy');
meta.setAttribute('content', `connect-src 'self' ${HTTPS_NOTSAMESITE_ORIGIN}`);
document.head.appendChild(meta);

promise_test(async t => {
  const uuid = token();
  const url = generateSetBeaconURL(uuid, {host: HTTPS_NOTSAMESITE_ORIGIN});
  fetchLater(url, {activateAfter: 0});

  await expectBeacon(uuid, {count: 1});
  t.done();
}, 'FetchLater allowed by CSP should succeed');
