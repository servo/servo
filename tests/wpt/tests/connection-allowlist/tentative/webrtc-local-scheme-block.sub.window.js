// META: script=/common/get-host-info.sub.js
// META: script=/content-security-policy/webrtc/webrtc.js
//
// The following tests assume the policy `Connection-Allowlist:
// (response-origin)` has been set. WebRTC has been blocked by default. Child
// frames with a local scheme (about:blank, about:srcdoc) inherit the parent's
// policy container, and should continue to block WebRTC, including when
// nested.

const WEBRTC_HELPER = '/content-security-policy/webrtc/webrtc.js';
// Loads webrtc.js into `win`'s own document, so `tryConnect` (and the
// RTCPeerConnection it uses) run in the child's realm, not the parent's.
function loadHelper(win) {
  return new Promise((resolve, reject) => {
    const script = win.document.createElement('script');
    script.src = WEBRTC_HELPER;
    script.onload = resolve;
    script.onerror = () => reject(new Error(`Failed to load ${WEBRTC_HELPER}`));
    win.document.head.appendChild(script);
  });
}

async function tryConnectIn(win) {
  await loadHelper(win);
  return win.tryConnect();
}

// Appends an iframe with no `src` to `doc`. The frame stays on its initial
// about:blank document, which is usable synchronously.
function addBlankFrame(t, doc) {
  const frame = doc.createElement('iframe');
  t.add_cleanup(() => frame.remove());
  doc.body.appendChild(frame);
  return frame;
}

// Appends an about:srcdoc iframe to `doc`, and waits for the srcdoc document
// to load (before that, `contentWindow` is still the initial about:blank
// document).
async function addSrcdocFrame(
    t, doc, srcdoc = '<!doctype html><body><p>Hello World</p></body>') {
  const frame = doc.createElement('iframe');
  t.add_cleanup(() => frame.remove());
  const loaded = new Promise(resolve => frame.onload = resolve);
  frame.srcdoc = srcdoc;
  doc.body.appendChild(frame);
  await loaded;
  return frame;
}

promise_test(async t => {
  assert_equals(await tryConnect(), 'blocked');
}, 'WebRTC is blocked in the top-level document.');

promise_test(async t => {
  const frame = addBlankFrame(t, document);
  assert_equals(await tryConnectIn(frame.contentWindow), 'blocked');
}, 'WebRTC is blocked in an about:blank child frame.');

promise_test(async t => {
  const frame = await addSrcdocFrame(t, document);
  assert_equals(await tryConnectIn(frame.contentWindow), 'blocked');
}, 'WebRTC is blocked in an about:srcdoc child frame.');

promise_test(async t => {
  const outer = addBlankFrame(t, document);
  const inner = addBlankFrame(t, outer.contentDocument);
  assert_equals(await tryConnectIn(inner.contentWindow), 'blocked');
}, 'WebRTC is blocked in a nested about:blank frame.');

promise_test(async t => {
  const outer = await addSrcdocFrame(t, document);
  const inner = await addSrcdocFrame(t, outer.contentDocument);
  assert_equals(await tryConnectIn(inner.contentWindow), 'blocked');
}, 'WebRTC is blocked in a nested about:srcdoc frame.');
