// META: script=../../../service-workers/service-worker/resources/test-helpers.sub.js
// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js
// META: script=resources/utils.js

const { REMOTE_HOST } = get_host_info();
const BASE_SCOPE = 'resources/basic.html?';

async function cleanup() {
  for (const iframe of document.querySelectorAll('.test-iframe')) {
    iframe.parentNode.removeChild(iframe);
  }

  for (const reg of await navigator.serviceWorker.getRegistrations()) {
    await reg.unregister();
  }
}

async function setupRegistration(t, scope) {
  await cleanup();
  const reg = await navigator.serviceWorker.register(
      'resources/sw-intercept-range.js', { scope });
  await wait_for_state(t, reg.installing, 'activated');
  return reg;
}

promise_test(async t => {
  const scope = BASE_SCOPE + Math.random();
  await setupRegistration(t, scope);
  const iframe = await with_iframe(scope);
  const w = iframe.contentWindow;

  const url = new URL('long-wav.py', w.location);
  url.searchParams.set('action', 'echo-range');

  const response = await w.fetch(url, { headers: { Range: 'bytes=0-10' } });

  assert_equals(response.status, 200, 'Response is served by the service worker');
  assert_equals(
      await response.text(), 'bytes=0-10',
      'Service worker intercepted the same-origin range request and saw its Range header');
}, 'Same-origin range request must be intercepted by the service worker');

promise_test(async t => {
  const scope = BASE_SCOPE + Math.random();
  await setupRegistration(t, scope);
  const iframe = await with_iframe(scope);
  const w = iframe.contentWindow;

  const url = new URL('long-wav.py', w.location);
  url.hostname = REMOTE_HOST;
  url.searchParams.set('action', 'echo-range');

  const response =
      await w.fetch(url, { mode: 'cors', headers: { Range: 'bytes=0-10' } });

  assert_equals(
      await response.text(), 'bytes=0-10',
      'Service worker intercepted the cross-origin CORS range request and saw its Range header');
}, 'Cross-origin CORS range request must be intercepted by the service worker');
