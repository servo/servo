// META: title=FetchLater: blocked by CSP
// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js
// META: script=/pending-beacon/resources/pending_beacon-helper.js
'use strict';

const {
  HTTPS_NOTSAMESITE_ORIGIN,
} = get_host_info();

// FetchLater requests blocked by Content Security Policy are rejected.
// https://w3c.github.io/webappsec-csp/#should-block-request

const meta = document.createElement('meta');
meta.setAttribute('http-equiv', 'Content-Security-Policy');
meta.setAttribute('content', 'connect-src \'self\'');
document.head.appendChild(meta);

promise_test(async t => {
  const uuid = token();
  const cspViolationUrl =
      generateSetBeaconURL(uuid, {host: HTTPS_NOTSAMESITE_ORIGIN});
  fetchLater(cspViolationUrl, {activateAfter: 0});

  await new Promise(
      resolve => window.addEventListener('securitypolicyviolation', e => {
        assert_equals(e.violatedDirective, 'connect-src');
        resolve();
      }));
  t.done();
}, 'FetchLater blocked by CSP should reject');
