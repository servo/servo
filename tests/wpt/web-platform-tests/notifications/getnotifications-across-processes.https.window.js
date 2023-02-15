// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js

// (Cannot use `global=serviceworker` because testdriver only supports window)

navigator.serviceWorker.addEventListener("message", async ev => {
  if (ev.data === "notification-create") {
    // (Scope used by service_worker_test)
    const scope = "scope" + window.location.pathname;
    const reg = await navigator.serviceWorker.getRegistration(scope);
    await reg.showNotification("Created from window");
    reg.active.postMessage("notification-created");
  }
});

promise_setup(() => {
  return test_driver.set_permission({ name: "notifications" }, "granted");
});

service_worker_test("getnotifications-sw.js", "Service worker test setup");
