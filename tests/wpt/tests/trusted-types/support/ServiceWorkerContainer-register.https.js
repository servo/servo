let test_setup_policy = trustedTypes.createPolicy("hurrayanythinggoes", {
  createScriptURL: x => x
});
importScripts(test_setup_policy.createScriptURL("/resources/testharness.js"));

// Determine worker type (for better logging)
let worker_type = "unknown";
if (this.DedicatedWorkerGlobalScope !== undefined) {
  worker_type = "dedicated worker";
} else if (this.SharedWorkerGlobalScope !== undefined) {
  worker_type = "shared worker";
} else if (this.ServiceWorkerGlobalScope !== undefined) {
  worker_type = "service worker";
}

let test_policy = trustedTypes.createPolicy("xxx", {
  createScriptURL: url => url.replace("play", "work")
});

promise_test(async t => {
  assert_true("navigator" in self);
  assert_true(self.navigator instanceof WorkerNavigator);
}, `WorkerNavigator exposed in ${worker_type}`);

if ('serviceWorker' in navigator) {

  // Passing a trusted type to register() should work.
  promise_test(async t => {
    let trusted_url = test_policy.createScriptURL("player.https.js");
    assert_true(this.trustedTypes.isScriptURL(trusted_url));
    const scope = `scope1/for/${worker_type}`;
    let reg = await self.navigator.serviceWorker.getRegistration(scope);
    if (reg) await reg.unregister();
    reg = await self.navigator.serviceWorker.register(trusted_url, {scope});
    await new Promise(r => reg.addEventListener("updatefound", r));
  }, `register() with TrustedScriptURL works in ${worker_type}`);

  // Passing a plain string to register() should fail.
  promise_test(async t => {
    let untrusted_url = "worker.https.js";
    const scope = `scope2/for/${worker_type}`;
    let reg = await self.navigator.serviceWorker.getRegistration(scope);
    if (reg) await reg.unregister();
    promise_rejects_js(t, TypeError, self.navigator.serviceWorker.register(untrusted_url, {scope}));
  }, `register() fails with plain string in ${worker_type}`);

  // Passing a plain string to register() should work after registering a
  // default policy.
  promise_test(async t => {
    trustedTypes.createPolicy("default", {
      createScriptURL: (url, _, sink) => {
        assert_equals(sink, "ServiceWorkerContainer register");
        return url.replace("play", "work");
      }
    });

    let untrusted_url = "player.https.js";
    const scope = `scope3/for/${worker_type}`;
    let reg = await self.navigator.serviceWorker.getRegistration(scope);
    if (reg) await reg.unregister();
    reg = await self.navigator.serviceWorker.register(untrusted_url, {scope});
    await new Promise(r => reg.addEventListener("updatefound", r));
  }, `register() fails with plain string in ${worker_type} with a default policy`);
}

done();
