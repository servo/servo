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

function awaitMessage(test, obj, id, maxTimeout) {
  return new Promise((resolve, reject) => {
    obj.addEventListener('message', function listener(event) {
      if (event.data.id !== id) return;
      obj.removeEventListener('message', listener);
      resolve(event.data);
    });
    if (maxTimeout)
      test.step_timeout(() => reject("awaiting message timed out"), maxTimeout);
  });
}

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
    const rangeBroadcast = awaitMessage(t, w.navigator.serviceWorker, rangeId, 1000);

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
