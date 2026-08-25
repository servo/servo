// META: title=`Origin.from(ExtendableMessageEvent)`
// META: global=serviceworker
// META: script=/common/get-host-info.sub.js

function WorkerActivationPromise() {
  return new Promise((resolve) => {
    if (registration.active) {
      resolve();
      return;
    }
    self.addEventListener('activate', () => { resolve(); });
  });
}

test(t => {
  const e = new ExtendableMessageEvent("message", { origin: get_host_info().ORIGIN });
  assert_throws_js(TypeError, _ => Origin.from(e));
}, "Constructed `ExtendableMessageEvent` objects have no real origins.");

promise_test(async t => {
  await WorkerActivationPromise();

  return new Promise(resolve => {
    self.addEventListener("message", e => {
      const origin = Origin.from(e);
      assert_true(!!origin);
      assert_false(origin.opaque);
      assert_true(origin.isSameOrigin(Origin.from(self)));
      resolve();
    });

    self.registration.active.postMessage({ type: "Hi" });
  });
}, "Posted `ExtendableMessageEvent` objects have origins.");
