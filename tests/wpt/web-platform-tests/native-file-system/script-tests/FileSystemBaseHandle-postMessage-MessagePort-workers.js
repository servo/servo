'use strict';

// This script depends on the following scripts:
//    /native-file-system/resources/messaging-helpers.js
//    /native-file-system/resources/messaging-blob-helpers.js
//    /native-file-system/resources/messaging-serialize-helpers.js
//    /native-file-system/resources/test-helpers.js
//    /service-workers/service-worker/resources/test-helpers.sub.js

directory_test(
    async (t, root_dir) => {
      const dedicated_worker =
          create_dedicated_worker(t, kDedicatedWorkerMessageTarget);
      await do_message_port_test(t, root_dir, /*target=*/ dedicated_worker);
    },
    'Send and receive messages using a message port in a dedicated ' +
        'worker.');

directory_test(
    async (t, root_dir) => {
      const scope = `${kServiceWorkerMessageTarget}` +
          '?post-message-to-message-port-with-file-handle';
      const registration =
          await create_service_worker(t, kServiceWorkerMessageTarget, scope);
      await do_message_port_test(
          t, root_dir, /*target=*/ registration.installing);
    },
    'Send and receive messages using a message port in a service ' +
        'worker.');

if (self.SharedWorker !== undefined) {
  directory_test(
      async (t, root_dir) => {
        const shared_worker = new SharedWorker(kSharedWorkerMessageTarget);
        shared_worker.port.start();
        await do_message_port_test(t, root_dir, /*target=*/ shared_worker.port);
      },
      'Send and receive messages using a message port in a shared ' +
          ' worker.');
}
