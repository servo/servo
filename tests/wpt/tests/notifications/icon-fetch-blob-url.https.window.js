// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=resources/helpers.js

async function createBlobUrl() {
  const response = await fetch("resources/icon.png");
  const buffer = await response.arrayBuffer();
  return URL.createObjectURL(new Blob([buffer], { type: "image/png" }));
}

let registration;

promise_setup(async () => {
  await trySettingPermission("granted");
  registration = await prepareActiveServiceWorker("noop-sw.js");
});

function notificationOptions(resource, url) {
  if (resource === "action icon") {
    return { actions: [{ action: "open", title: "Open", icon: url }] };
  }

  return { [resource]: url };
}

for (const resource of ["icon", "image", "badge", "action icon"]) {
  promise_test(async t => {
    const url = await createBlobUrl();

    t.add_cleanup(() => {
      URL.revokeObjectURL(url);
      closeAllNotifications();
    });

    const notificationPromise = registration.showNotification(
      `Notification with blob ${resource}`,
      notificationOptions(resource, url)
    );

    // The notification fetch must retain the blob after the page revokes its
    // own reference to the object URL.
    URL.revokeObjectURL(url);

    await notificationPromise;
  }, `showNotification keeps a blob URL alive for a ${resource}`);
}

for (const resource of ["icon", "image", "badge", "action icon"]) {
  promise_test(async t => {
    const url = await createBlobUrl();
    URL.revokeObjectURL(url);

    t.add_cleanup(closeAllNotifications);

    // A resource that was revoked before the request was created cannot be
    // fetched. The notification should still finish without that resource.
    await registration.showNotification(
      `Notification with revoked blob ${resource}`,
      notificationOptions(resource, url)
    );
  }, `showNotification handles a revoked blob URL for a ${resource}`);
}
