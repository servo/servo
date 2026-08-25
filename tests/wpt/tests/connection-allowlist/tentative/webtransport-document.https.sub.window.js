// META: script=/common/get-host-info.sub.js
//
// The following tests assume the policy
// `Connection-Allowlist: (response-origin
// "https://{{host}}:{{ports[webtransport-h3][0]}}")` has been set. The WPT
// WebTransport server runs on a different port than HTTPS, so the allowlist
// explicitly includes the same-host WebTransport origin.

const wt_port = '{{ports[webtransport-h3][0]}}';

function webtransport_test(host, expectation, description) {
  promise_test(async t => {
    const url = `https://${host}:${
        wt_port}/webtransport/handlers/custom-response.py?:status=200`;
    const wt = new WebTransport(url);

    const result = await wt.ready.then(() => {
      wt.close();
      return 'open';
    }, (e) => 'error');

    assert_equals(
        result, expectation,
        `WebTransport to ${host} should ${
            expectation === 'open' ? 'connect' : 'be blocked'}.`);
  }, description);
}

// Same-origin WebTransport should succeed (allowlisted via explicit pattern).
webtransport_test(
    '{{hosts[][]}}', 'open', 'Same-origin WebTransport succeeds.');

// Same-site but cross-origin subdomains should fail.
webtransport_test(
    '{{hosts[][www]}}', 'error',
    'Cross-origin same-site WebTransport (www) is blocked.');

webtransport_test(
    '{{hosts[][www1]}}', 'error',
    'Cross-origin same-site WebTransport (www1) is blocked.');

// Cross-site origins should fail.
webtransport_test(
    '{{hosts[alt][]}}', 'error', 'Cross-site WebTransport is blocked.');

webtransport_test(
    '{{hosts[alt][www]}}', 'error',
    'Cross-site WebTransport (www subdomain) is blocked.');
