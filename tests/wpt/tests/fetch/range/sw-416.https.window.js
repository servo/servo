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
  await setupRegistration(t, scope);
  const iframe = await with_iframe(scope);
  const w = iframe.contentWindow;
  const id = Math.random() + '';
  const storedRangeResponse = awaitMessage(w.navigator.serviceWorker, id);

  const url = new URL('partial-script.py', w.location);
  url.searchParams.set('require-range', '1');
  url.searchParams.set('range-not-satisfiable', '1');
  url.searchParams.set('type', 'image/png');
  url.searchParams.set('action', 'store-ranged-response');
  url.searchParams.set('id', id);
  url.hostname = REMOTE_HOST;

  appendAudio(w.document, url);

  await storedRangeResponse;

  const fetchPromise = w.fetch('?action=use-stored-ranged-response', { mode: 'no-cors' });
  await promise_rejects_js(t, w.TypeError, fetchPromise);

  const loadImagePromise = loadImage('?action=use-stored-ranged-response', { doc: w.document });
  await promise_rejects_js(t, Error, loadImagePromise);
}, `416 response not allowed following no-cors ranged request`);
