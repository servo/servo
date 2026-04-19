// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=resources/utils.js
// META: timeout=long
//
// Tests that injecting resource hint <link> elements into same-origin iframe
// contentWindows does not bypass Connection-Allowlist enforcement. The policy
// should apply based on the iframe document's policy (inherited for about:blank,
// or from its own response headers for same-origin navigated iframes).

const port = get_host_info().HTTP_PORT_ELIDED;
const BLOCKED_ORIGIN = 'http://{{hosts[][www]}}' + port;

// Helper: inject a <link rel="prefetch"> into an iframe's contentWindow
// and verify the request does not reach the server.
function iframe_injection_test(iframe_setup, description) {
  promise_test(async t => {
    const key = token();
    const value = 'leaked';
    const params = new URLSearchParams();
    params.set('key', key);
    params.set('value', value);

    const url = `${BLOCKED_ORIGIN}${STORE_URL}?${params.toString()}`;

    // Create and configure the iframe.
    const iframe = await iframe_setup(t);

    // Inject a <link rel="prefetch"> into the iframe's document via
    // contentWindow to verify enforcement applies to injected content.
    iframe.contentWindow.document.head.innerHTML =
        `<link rel="prefetch" href="${url}">`;

    // If Connection-Allowlist enforcement is bypassed, the prefetch
    // request will reach the server and store the value. Verify it
    // does not.
    const result = await Promise.race([
      new Promise(r => t.step_timeout(r, 2000)),
      nextValueFromServer(key)
    ]);
    assert_equals(result, undefined,
        `Prefetch injected into iframe should be blocked.`);
  }, description);
}

// --- about:blank iframe (inherits parent's policy) ---

iframe_injection_test(async (t) => {
  const iframe = document.createElement('iframe');
  // about:blank is the default; the iframe inherits the parent's
  // Connection-Allowlist policy per spec.
  document.body.appendChild(iframe);
  t.add_cleanup(() => iframe.remove());

  // Small delay to ensure the iframe's document is ready.
  await new Promise(r => t.step_timeout(r, 100));
  return iframe;
}, 'Injecting <link rel="prefetch"> into about:blank iframe contentWindow ' +
   'must be blocked by inherited Connection-Allowlist.');

// Variant using explicit src="about:" for the iframe.
iframe_injection_test(async (t) => {
  const iframe = document.createElement('iframe');
  iframe.src = 'about:';
  document.body.appendChild(iframe);
  t.add_cleanup(() => iframe.remove());

  await new Promise(r => t.step_timeout(r, 100));
  return iframe;
}, 'Injecting <link rel="prefetch"> into about: iframe contentWindow ' +
   'must be blocked by inherited Connection-Allowlist.');

// Variant using src=" about:" (leading space) for the iframe.
// Browsers typically strip leading whitespace from URLs before navigation,
// so this likely resolves to about: and behaves identically, but we test
// it explicitly to guard against edge cases in URL parsing.
iframe_injection_test(async (t) => {
  const iframe = document.createElement('iframe');
  iframe.src = ' about:';
  document.body.appendChild(iframe);
  t.add_cleanup(() => iframe.remove());

  await new Promise(r => t.step_timeout(r, 100));
  return iframe;
}, 'Injecting <link rel="prefetch"> into " about:" (space-prefixed) iframe ' +
   'contentWindow must be blocked by inherited Connection-Allowlist.');

// --- Same-origin iframe with its own Connection-Allowlist ---
// Uses a helper page served with Connection-Allowlist headers
// to ensure the iframe's document has the policy.

iframe_injection_test(async (t) => {
  const iframe = document.createElement('iframe');
  iframe.src = 'resources/blank-with-allowlist.html';
  document.body.appendChild(iframe);
  t.add_cleanup(() => iframe.remove());

  // Wait for the iframe to fully load its document.
  await new Promise(resolve => { iframe.onload = resolve; });

  // Clear existing head content before injecting.
  iframe.contentWindow.document.head.innerHTML = '';
  return iframe;
}, 'Injecting <link rel="prefetch"> into same-origin iframe (with its own ' +
   'Connection-Allowlist) contentWindow must be blocked.');

// --- Additional variant: createElement injection instead of innerHTML ---
// Tests that the bypass is not specific to innerHTML parsing.

promise_test(async t => {
  const key = token();
  const value = 'leaked';
  const params = new URLSearchParams();
  params.set('key', key);
  params.set('value', value);

  const url = `${BLOCKED_ORIGIN}${STORE_URL}?${params.toString()}`;

  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  t.add_cleanup(() => iframe.remove());

  await new Promise(r => t.step_timeout(r, 100));

  // Inject via createElement instead of innerHTML.
  const link = iframe.contentWindow.document.createElement('link');
  link.rel = 'prefetch';
  link.href = url;
  iframe.contentWindow.document.head.appendChild(link);

  const result = await Promise.race([
    new Promise(r => t.step_timeout(r, 2000)),
    nextValueFromServer(key)
  ]);
  assert_equals(result, undefined,
      'Prefetch via createElement in iframe should be blocked.');
}, 'Injecting <link rel="prefetch"> via createElement into about:blank ' +
   'iframe contentWindow must be blocked by inherited Connection-Allowlist.');
