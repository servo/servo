// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js

function unregisterAllServiceWorker() {
  return navigator.serviceWorker.getRegistrations().then(registrations => {
    return Promise.all(registrations.map(r => r.unregister()));
  });
}

async function prepareActiveServiceWorker(script) {
  await unregisterAllServiceWorker();
  const reg = await navigator.serviceWorker.register(script);
  add_completion_callback(() => reg.unregister());
  await navigator.serviceWorker.ready;
  return reg;
}

let registration;

promise_setup(async () => {
  registration = await prepareActiveServiceWorker("offscreencanvas.transferrable.sw.js");
});

promise_test(async () => {
  const canvas = new OffscreenCanvas(100, 100);
  registration.active.postMessage({ canvas }, { transfer: [canvas] });
  const data = await new Promise(resolve => navigator.serviceWorker.addEventListener("message", ev => {
    resolve(ev.data);
  }, { once: true }));
  assert_equals(data.constructorName, "OffscreenCanvas", "Should get OffscreenCanvas from the window")
  assert_true(data.canvas instanceof OffscreenCanvas, "Should get OffscreenCanvas from the service worker");
}, "Sending and receiving OffscreenCanvas between window and service worker");
