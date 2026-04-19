// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=resources/utils.js
// META: timeout=long
//
// Tests that Connection-Allowlist: (response-origin) blocks various
// subresource fetch types to non-allowlisted origins. These cover
// element types beyond <link rel="prefetch"> that initiate network
// requests: forms, object/embed, CSS, srcset, and multi-value rel.

const port = get_host_info().HTTP_PORT_ELIDED;
const BLOCKED_ORIGIN = 'http://{{hosts[][www]}}' + port;
const SAME_ORIGIN = 'http://{{hosts[][]}}' + port;

// Helper: verify that a request to the blocked origin does NOT arrive
// at the server's key-value store.
function blocked_request_test(setup_fn, description) {
  promise_test(async t => {
    const key = token();
    const value = 'leaked';
    const params = new URLSearchParams();
    params.set('key', key);
    params.set('value', value);

    const url = `${BLOCKED_ORIGIN}${STORE_URL}?${params.toString()}`;

    setup_fn(t, url);

    const result = await Promise.race([
      new Promise(r => t.step_timeout(r, 2000)),
      nextValueFromServer(key)
    ]);
    assert_equals(result, undefined,
        `Request should be blocked by Connection-Allowlist.`);
  }, description);
}

// --- Same-origin control test ---
// Verify that same-origin requests succeed, confirming that failures in
// the tests below are due to the origin being blocked, not something
// else about the fetch URL or key-value store.
promise_test(async t => {
  const key = token();
  const value = 'control';
  const params = new URLSearchParams();
  params.set('key', key);
  params.set('value', value);

  const url = `${SAME_ORIGIN}${STORE_URL}?${params.toString()}`;

  const link = document.createElement('link');
  link.rel = 'prefetch';
  link.href = url;
  document.head.appendChild(link);
  t.add_cleanup(() => link.remove());

  const result = await nextValueFromServer(key);
  assert_equals(result, value,
      `Same-origin prefetch should succeed and store the value.`);
}, 'Same-origin prefetch control test succeeds (verifies test infrastructure).');

// --- Multi-value rel attribute ---
// The HTML spec allows multiple space-separated rel values. These tests
// verify that when a dangerous rel type (prefetch, etc.) appears alongside
// other values, Connection-Allowlist still blocks the resulting request.

blocked_request_test((t, url) => {
  const link = document.createElement('link');
  link.rel = 'alternate prefetch';
  link.href = url;
  document.head.appendChild(link);
  t.add_cleanup(() => link.remove());
}, 'Multi-value rel="alternate prefetch" to blocked origin must be blocked.');

blocked_request_test((t, url) => {
  const link = document.createElement('link');
  link.rel = 'prefetch stylesheet';
  link.href = url;
  document.head.appendChild(link);
  t.add_cleanup(() => link.remove());
}, 'Multi-value rel="prefetch stylesheet" to blocked origin must be blocked.');

// --- Form cross-origin action ---
// Form submission to a non-allowlisted origin should be blocked.

blocked_request_test((t, url) => {
  const form = document.createElement('form');
  form.method = 'GET';
  form.action = url;
  const input = document.createElement('input');
  input.type = 'hidden';
  input.name = 'data';
  input.value = 'exfil';
  form.appendChild(input);
  document.body.appendChild(form);
  t.add_cleanup(() => form.remove());

  // Submit inside an iframe to avoid navigating the test page.
  const iframe = document.createElement('iframe');
  iframe.name = 'form-target-' + token();
  document.body.appendChild(iframe);
  t.add_cleanup(() => iframe.remove());
  form.target = iframe.name;
  form.submit();
}, 'Form with cross-origin action to blocked origin must be blocked.');

// --- formaction attribute ---

blocked_request_test((t, url) => {
  const form = document.createElement('form');
  form.method = 'GET';
  form.action = SAME_ORIGIN + '/';
  const button = document.createElement('button');
  button.type = 'submit';
  button.formAction = url;
  form.appendChild(button);
  document.body.appendChild(form);
  t.add_cleanup(() => form.remove());

  const iframe = document.createElement('iframe');
  iframe.name = 'formaction-target-' + token();
  document.body.appendChild(iframe);
  t.add_cleanup(() => iframe.remove());
  form.target = iframe.name;
  button.click();
}, 'Button with cross-origin formaction to blocked origin must be blocked.');

// --- Object and Embed ---

blocked_request_test((t, url) => {
  const obj = document.createElement('object');
  obj.data = url;
  document.body.appendChild(obj);
  t.add_cleanup(() => obj.remove());
}, '<object data="cross-origin"> to blocked origin must be blocked.');

blocked_request_test((t, url) => {
  const embed = document.createElement('embed');
  embed.src = url;
  document.body.appendChild(embed);
  t.add_cleanup(() => embed.remove());
}, '<embed src="cross-origin"> to blocked origin must be blocked.');

// --- CSS url() exfiltration ---

blocked_request_test((t, url) => {
  const div = document.createElement('div');
  div.style.backgroundImage = `url("${url}")`;
  document.body.appendChild(div);
  t.add_cleanup(() => div.remove());
}, 'CSS background-image: url() to blocked origin must be blocked.');

blocked_request_test((t, url) => {
  const style = document.createElement('style');
  style.textContent = `@import url("${url}");`;
  document.head.appendChild(style);
  t.add_cleanup(() => style.remove());
}, 'CSS @import url() to blocked origin must be blocked.');

// --- Image srcset ---

blocked_request_test((t, url) => {
  const img = document.createElement('img');
  img.srcset = `${url} 1x`;
  document.body.appendChild(img);
  t.add_cleanup(() => img.remove());
}, '<img srcset="cross-origin"> to blocked origin must be blocked.');

// --- Video poster ---

blocked_request_test((t, url) => {
  const video = document.createElement('video');
  video.poster = url;
  document.body.appendChild(video);
  t.add_cleanup(() => video.remove());
}, '<video poster="cross-origin"> to blocked origin must be blocked.');
