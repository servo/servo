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
  const origin = Origin.from(e);
  assert_true(!!origin, "It's not null!");
  assert_false(origin.opaque, "It's not opaque!");
  assert_true(origin.isSameOrigin(Origin.from(self)), "It's same-origin with an Origin!");
}, "Constructed `ExtendableMessageEvent` objects have origins.");


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
