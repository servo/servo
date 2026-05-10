// META: script=/common/get-host-info.sub.js
//
// The following tests assume the policy
// `Connection-Allowlist: (response-origin "http://{{host}}:{{ports[ws][0]}}")`
// has been set. The WPT WebSocket server runs on a different port than HTTP,
// so the allowlist explicitly includes the same-host WebSocket origin.

const ws_port = "{{ports[ws][0]}}";

function websocket_test(host, expectation, description) {
  promise_test(async t => {
    const url = `ws://${host}:${ws_port}/echo`;
    const ws = new WebSocket(url);

    const result = await new Promise(resolve => {
      ws.onopen = () => { ws.close(); resolve('open'); };
      ws.onerror = () => resolve('error');
    });

    assert_equals(result, expectation,
        `WebSocket to ${host} should ${expectation === 'open' ? 'connect' : 'be blocked'}.`);
  }, description);
}

// Same-origin WebSocket should succeed (allowlisted via explicit pattern).
websocket_test(
  "{{hosts[][]}}",
  "open",
  "Same-origin WebSocket succeeds."
);

// Same-site but cross-origin subdomains should fail.
websocket_test(
  "{{hosts[][www]}}",
  "error",
  "Cross-origin same-site WebSocket (www) is blocked."
);

websocket_test(
  "{{hosts[][www1]}}",
  "error",
  "Cross-origin same-site WebSocket (www1) is blocked."
);

// Cross-site origins should fail.
websocket_test(
  "{{hosts[alt][]}}",
  "error",
  "Cross-site WebSocket is blocked."
);

websocket_test(
  "{{hosts[alt][www]}}",
  "error",
  "Cross-site WebSocket (www subdomain) is blocked."
);
