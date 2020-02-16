'use strict';

// This script depends on the following scripts:
//    /native-file-system/resources/messaging-helpers.js
//    /native-file-system/resources/messaging-blob-helpers.js
//    /native-file-system/resources/messaging-serialize-helpers.js
//    /native-file-system/resources/test-helpers.js
//    /service-workers/service-worker/resources/test-helpers.sub.js

directory_test(async (t, root_dir) => {
  const dedicated_worker =
      create_dedicated_worker(t, kDedicatedWorkerMessageTarget);
  await do_post_message_test(
      t, root_dir, /*receiver=*/ dedicated_worker,
      /*target=*/ dedicated_worker);
}, 'Send and receive messages using a dedicated worker.');

directory_test(async (t, root_dir) => {
  const scope = `${kServiceWorkerMessageTarget}?post-message-with-file-handle`;
  const registration =
      await create_service_worker(t, kServiceWorkerMessageTarget, scope);
  await do_post_message_test(
      t, root_dir, /*receiver=*/ navigator.serviceWorker,
      /*target=*/ registration.installing);
}, 'Send and receive messages using a service worker.');

if (self.SharedWorker !== undefined) {
  directory_test(async (t, root_dir) => {
    const shared_worker = new SharedWorker(kSharedWorkerMessageTarget);
    shared_worker.port.start();
    await do_post_message_test(
        t, root_dir, /*receiver=*/ shared_worker.port,
        /*target=*/ shared_worker.port);
  }, 'Send and receive messages using a shared worker.');
}
