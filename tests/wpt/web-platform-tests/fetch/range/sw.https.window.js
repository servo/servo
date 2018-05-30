// META: script=../../../service-workers/service-worker/resources/test-helpers.sub.js
// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js
// META: script=resources/utils.js

const { REMOTE_HOST } = get_host_info();
const SCOPE = 'resources/basic.html' + Math.random();

function appendAudio(document, url) {
  const audio = document.createElement('audio');
  audio.muted = true;
  audio.src = url;
  audio.preload = true;
  document.body.appendChild(audio);
}

async function cleanup() {
  for (const iframe of document.querySelectorAll('.test-iframe')) {
    iframe.parentNode.removeChild(iframe);
  }

  const reg = await navigator.serviceWorker.getRegistration(SCOPE);
  if (reg) await reg.unregister();
}

async function setupRegistration(t) {
  await cleanup();
  const reg = await navigator.serviceWorker.register('resources/range-sw.js', { scope: SCOPE });
  await wait_for_state(t, reg.installing, 'activated');
  return reg;
}

function awaitMessage(obj, id) {
  return new Promise(resolve => {
    obj.addEventListener('message', function listener(event) {
      if (event.data.id !== id) return;
      obj.removeEventListener('message', listener);
      resolve();
    });
  });
}

promise_test(async t => {
  const reg = await setupRegistration(t);
  const iframe = await with_iframe(SCOPE);
  const w = iframe.contentWindow;

  // Trigger a cross-origin range request using media
  const url = new URL('long-wav.py?action=range-header-filter-test', w.location);
  url.hostname = REMOTE_HOST;
  appendAudio(w.document, url);

  // See rangeHeaderFilterTest in resources/range-sw.js
  await fetch_tests_from_worker(reg.active);
}, `Defer range header filter tests to service worker`);

promise_test(async t => {
  const reg = await setupRegistration(t);
  const iframe = await with_iframe(SCOPE);
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
  await setupRegistration(t);
  const iframe = await with_iframe(SCOPE);
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
  promise_rejects(t, new TypeError(), fetchPromise);

  // Script loading should error too
  const loadScriptPromise = loadScript('?action=use-stored-ranged-response', { doc: w.document });
  promise_rejects(t, new Error(), loadScriptPromise);

  await loadScriptPromise.catch(() => {});

  assert_false(!!w.scriptExecuted, `Partial response shouldn't be executed`);
}, `Ranged response not allowed following no-cors ranged request`);

promise_test(async t => {
  await setupRegistration(t);
  const iframe = await with_iframe(SCOPE);
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
