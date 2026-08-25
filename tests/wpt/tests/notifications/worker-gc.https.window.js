// META: title=An active notification should prevent worker cycle collection
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/common/gc.js
// META: script=resources/helpers.js

promise_setup(async () => {
  await trySettingPermission("granted");
});

promise_test(async t => {
  let worker = new Worker(URL.createObjectURL(new Blob([`
    const n = new Notification("foo");
    onmessage = () => n.close();
    n.onclose = () => self.postMessage("closed");
    postMessage("ready");
  `])));
  await new Promise(resolve => {
    worker.addEventListener("message", ev => {
      if (ev.data === "ready") {
        resolve();
      }
    }, { once: true });
  });
  const weakref = new WeakRef(worker);
  worker = null;

  t.add_cleanup(() => {
    weakref.deref()?.postMessage("close");
  });

  await garbageCollect();
  assert_true(!!weakref.deref());
});
