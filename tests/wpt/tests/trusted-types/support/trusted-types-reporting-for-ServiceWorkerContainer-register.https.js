const policy = trustedTypes.createPolicy("dummy", { createScriptURL: x => x });
let worker_url = "worker.https.js";
let scope = `scope1/`;

if (getGlobalThisStr().includes("Window")) {
  worker_url = `support/${worker_url}`;
  scope = `support/${scope}`;
}

promise_test(async t => {
  await no_trusted_type_violation_for(async _ => {
    let reg = await self.navigator.serviceWorker.getRegistration(scope);
    if (reg) await reg.unregister();
    reg = await self.navigator.serviceWorker.register(policy.createScriptURL(worker_url), {scope});
    await new Promise(r => reg.addEventListener("updatefound", r));
  });
}, "No violation reported for ServiceWorkerContainer register with TrustedScriptURL.");

promise_test(async t => {
  let violation = await trusted_type_violation_for(TypeError, async _ => {
    let reg = await self.navigator.serviceWorker.getRegistration(scope);
    if (reg) await reg.unregister();
    await self.navigator.serviceWorker.register(worker_url, {scope});
  });
  assert_equals(violation.blockedURI, "trusted-types-sink");
  assert_equals(violation.sample, `ServiceWorkerContainer register|${clipSampleIfNeeded(worker_url)}`);
}, "Violation report for ServiceWorkerContainer register with plain string.");
