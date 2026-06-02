// META: title=FetchLater: redirect blocked by CSP
// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js
// META: script=/fetch/fetch-later/resources/fetch-later-helper.js
// META: timeout=long

'use strict';

const {
  HTTPS_NOTSAMESITE_ORIGIN,
} = get_host_info();

// FetchLater requests redirect to URL blocked by Content Security Policy.
// https://w3c.github.io/webappsec-csp/#should-block-request

const meta = document.createElement('meta');
meta.setAttribute('http-equiv', 'Content-Security-Policy');
meta.setAttribute('content', 'connect-src \'self\'');
document.head.appendChild(meta);

promise_test(async t => {
  const uuid = token();
  const cspViolationUrl =
      generateSetBeaconURL(uuid, {host: HTTPS_NOTSAMESITE_ORIGIN});
  const url =
      `/common/redirect.py?location=${encodeURIComponent(cspViolationUrl)}`;
  fetchLater(url, {activateAfter: 0});

  // TODO(crbug.com/1465781): redirect csp check is handled in browser, of which
  // result cannot be populated to renderer at this moment.
  await expectBeacon(uuid, {count: 0});
  t.done();
}, 'FetchLater redirect blocked by CSP should reject');
