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
  const reg = await navigator.serviceWorker.register('resources/range-sw.js', { scope });
  await wait_for_state(t, reg.installing, 'activated');
  return reg;
}

function awaitMessage(obj, id) {
  return new Promise(resolve => {
    obj.addEventListener('message', function listener(event) {
      if (event.data.id !== id) return;
      obj.removeEventListener('message', listener);
      resolve(event.data);
    });
  });
}

promise_test(async t => {
  const scope = BASE_SCOPE + Math.random();
  const reg = await setupRegistration(t, scope);
  const iframe = await with_iframe(scope);
  const w = iframe.contentWindow;

  // Trigger a cross-origin range request using media
  const url = new URL('long-wav.py?action=range-header-filter-test', w.location);
  url.hostname = REMOTE_HOST;
  appendAudio(w.document, url);

  // See rangeHeaderFilterTest in resources/range-sw.js
  await fetch_tests_from_worker(reg.active);
}, `Defer range header filter tests to service worker`);

promise_test(async t => {
  const scope = BASE_SCOPE + Math.random();
  const reg = await setupRegistration(t, scope);
  const iframe = await with_iframe(scope);
  const w = iframe.contentWindow;

  // Trigger a cross-origin range request using media
  const url = new URL('long-wav.py', w.location);
  url.searchParams.set('action', 'range-header-passthrough-test');
  url.searchParams.set('range-received-key', token());
  url.hostname = REMOTE_HOST;
  appendAudio(w.document, url);

  // See rangeHeaderPassthroughTest in resources/range-sw.js
  await fetch_tests_from_worker(reg.active);
}, `Defer range header passthrough tests to service worker`);

promise_test(async t => {
  const scope = BASE_SCOPE + Math.random();
  await setupRegistration(t, scope);
  const iframe = await with_iframe(scope);
  const w = iframe.contentWindow;
  const id = Math.random() + '';
  const storedRangeResponse = awaitMessage(w.navigator.serviceWorker, id);

  // Trigger a cross-origin range request using media
  const url = new URL('partial-script.py', w.location);
  url.searchParams.set('require-range', '1');
  url.searchParams.set('action', 'store-ranged-response');
  url.searchParams.set('id', id);
  url.hostname = REMOTE_HOST;

  appendAudio(w.document, url);

  await storedRangeResponse;

  // Fetching should reject
  const fetchPromise = w.fetch('?action=use-stored-ranged-response', { mode: 'no-cors' });
  await promise_rejects_js(t, w.TypeError, fetchPromise);

  // Script loading should error too
  const loadScriptPromise = loadScript('?action=use-stored-ranged-response', { doc: w.document });
  await promise_rejects_js(t, Error, loadScriptPromise);

  await loadScriptPromise.catch(() => {});

  assert_false(!!w.scriptExecuted, `Partial response shouldn't be executed`);
}, `Ranged response not allowed following no-cors ranged request`);

promise_test(async t => {
  const scope = BASE_SCOPE + Math.random();
  await setupRegistration(t, scope);
  const iframe = await with_iframe(scope);
  const w = iframe.contentWindow;
  const id = Math.random() + '';
  const storedRangeResponse = awaitMessage(w.navigator.serviceWorker, id);

  // Trigger a range request using media
  const url = new URL('partial-script.py', w.location);
  url.searchParams.set('require-range', '1');
  url.searchParams.set('action', 'store-ranged-response');
  url.searchParams.set('id', id);

  appendAudio(w.document, url);

  await storedRangeResponse;

  // This should not throw
  await w.fetch('?action=use-stored-ranged-response');

  // This shouldn't throw either
  await loadScript('?action=use-stored-ranged-response', { doc: w.document });

  assert_true(w.scriptExecuted, `Partial response should be executed`);
}, `Non-opaque ranged response executed`);

promise_test(async t => {
  const scope = BASE_SCOPE + Math.random();
  await setupRegistration(t, scope);
  const iframe = await with_iframe(scope);
  const w = iframe.contentWindow;
  const fetchId = Math.random() + '';
  const fetchBroadcast = awaitMessage(w.navigator.serviceWorker, fetchId);
  const audioId = Math.random() + '';
  const audioBroadcast = awaitMessage(w.navigator.serviceWorker, audioId);

  const url = new URL('long-wav.py', w.location);
  url.searchParams.set('action', 'broadcast-accept-encoding');
  url.searchParams.set('id', fetchId);

  await w.fetch(url, {
    headers: { Range: 'bytes=0-10' }
  });

  assert_equals((await fetchBroadcast).acceptEncoding, null, "Accept-Encoding should not be set for fetch");

  url.searchParams.set('id', audioId);
  appendAudio(w.document, url);

  assert_equals((await audioBroadcast).acceptEncoding, null, "Accept-Encoding should not be set for media");
}, `Accept-Encoding should not appear in a service worker`);

promise_test(async t => {
  const scope = BASE_SCOPE + Math.random();
  await setupRegistration(t, scope);
  const iframe = await with_iframe(scope);
  const w = iframe.contentWindow;
  const length = 100;
  const count = 3;
  const counts = {};

  // test a single range request size
  async function testSizedRange(size, partialResponseCode) {
    const rangeId = Math.random() + '';
    const rangeBroadcast = awaitMessage(w.navigator.serviceWorker, rangeId);

    // Create a bogus audio element to trick the browser into sending
    // cross-origin range requests that can be manipulated by the service worker.
    const sound_url = new URL('partial-text.py', w.location);
    sound_url.hostname = REMOTE_HOST;
    sound_url.searchParams.set('action', 'record-media-range-request');
    sound_url.searchParams.set('length', length);
    sound_url.searchParams.set('size', size);
    sound_url.searchParams.set('partial', partialResponseCode);
    sound_url.searchParams.set('id', rangeId);
    sound_url.searchParams.set('type', 'audio/mp4');
    appendAudio(w.document, sound_url);

    // wait for the range requests to happen
    await rangeBroadcast;

    // Create multiple preload requests and count the number of resource timing
    // entries that get created to make sure 206 and 416 range responses are treated
    // the same.
    const url = new URL('partial-text.py', w.location);
    url.searchParams.set('action', 'use-media-range-request');
    url.searchParams.set('size', size);
    url.searchParams.set('type', 'audio/mp4');
    counts['size' + size] = 0;
    for (let i = 0; i < count; i++) {
      await preloadImage(url, { doc: w.document });
    }
  }

  // Test range requests from 1 smaller than the correct size to 1 larger than
  // the correct size to exercise the various permutations using the default 206
  // response code for successful range requests.
  for (let size = length - 1; size <= length + 1; size++) {
    await testSizedRange(size, '206');
  }

  // Test a successful range request using a 200 response.
  await testSizedRange(length - 2, '200');

  // Check the resource timing entries and count the reported number of fetches of each type
  const resources = w.performance.getEntriesByType("resource");
  for (const entry of resources) {
    const url = new URL(entry.name);
    if (url.searchParams.has('action') &&
        url.searchParams.get('action') == 'use-media-range-request' &&
        url.searchParams.has('size')) {
      counts['size' + url.searchParams.get('size')]++;
    }
  }

  // Make sure there are a non-zero number of preload requests and they are all the same
  let counts_valid = true;
  const first = 'size' + (length - 2);
  for (let size = length - 2; size <= length + 1; size++) {
    let key = 'size' + size;
    if (!(key in counts) || counts[key] <= 0 || counts[key] != counts[first]) {
      counts_valid = false;
      break;
    }
  }

  assert_true(counts_valid, `Opaque range request preloads were different for error and success`);
}, `Opaque range preload successes and failures should be indistinguishable`);
