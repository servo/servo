// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/notifications/resources/helpers.js

promise_setup(async () => {
  await trySettingPermission("granted");
  await prepareActiveServiceWorker("push-sw.js");
});

promise_test(async () => {
  // (no `sender` URL parameter needed as this worker is the only sender)
  const worker = new Worker("./resources/permission-worker.js", { type: "module" });
  const { promise, resolve } = Promise.withResolvers();
  worker.onmessage = ev => {
    if ("subscribed" in ev.data) {
      resolve(ev.data)
    }
  };

  const result = await promise;
  assert_true(result.subscribed, `subscription should happen`);
}, "Subscribing within a worker");
