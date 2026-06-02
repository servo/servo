// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/helpers.js

let registration;

promise_setup(async () => {
  await trySettingPermission("granted");
  registration = await prepareActiveServiceWorker("icon-fetch-sw.js");
  await new Promise(r => navigator.serviceWorker.addEventListener("controllerchange", r, { once: true }));
});

promise_test(async t => {
  const iconUrl = new URL("resources/icon.png", location.href).toString();

  const { promise, resolve } = Promise.withResolvers();
  navigator.serviceWorker.addEventListener("message", async ev => {
    if (ev.data.url === iconUrl) {
      resolve();
    }
  }, { signal: t.signal });

  await registration.showNotification("new Notification", {
    icon: iconUrl
  });
  t.add_cleanup(closeAllNotifications);

  await promise;
}, "Icon fetch should cause a corresponding fetch event in the service worker");
