'use strict';

// This script depends on the following scripts:
//    /fs/resources/messaging-helpers.js
//    /fs/resources/messaging-serialize-helpers.js
//    /fs/resources/test-helpers.js
//    /service-workers/service-worker/resources/test-helpers.sub.js

// Sets up a new broadcast channel in |target|. Posts a message instructing
// |target| to open the broadcast channel using |broadcast_channel_name|.
async function create_broadcast_channel(
  test, broadcast_channel_name, receiver, target, target_origin) {
  target.postMessage(
    { type: 'create-broadcast-channel', broadcast_channel_name },
    { targetOrigin: target_origin });
  const event_watcher = new EventWatcher(test, receiver, 'message');

  // Wait until |target| is listening to the broad cast channel.
  const message_event = await event_watcher.wait_for('message');
  assert_equals(message_event.data.type, 'broadcast-channel-created',
    'The message target must receive a "broadcast-channel-created" message ' +
    'response.');
}

// This test is very similar to 'FileSystemBaseHandle-postMessage.js'. It
// starts by creating three message targets for the broadcast channel:
// an iframe, dedicated worker and a service worker. After setup, an array
// of FileSystemHandles is sent across the broadcast channel. The test
// expects three responses -- one from each message target.
directory_test(
    async (t, root) => {
      const broadcast_channel_name = 'file-system-file-handle-channel';
      const broadcast_channel = new BroadcastChannel(broadcast_channel_name);
      const broadcast_channel_event_watcher =
          new EventWatcher(t, broadcast_channel, 'message');

      const iframe = await add_iframe(t, {src: kDocumentMessageTarget});
      await create_broadcast_channel(
          t, broadcast_channel_name, self, iframe.contentWindow, '*');

      const scope = `${kServiceWorkerMessageTarget}` +
          '?post-message-to-broadcast-channel-with-file-handle';

      const registration =
          await create_service_worker(t, kServiceWorkerMessageTarget, scope);

      await create_broadcast_channel(
          t, broadcast_channel_name, navigator.serviceWorker,
          registration.installing);

      const dedicated_worker =
          create_dedicated_worker(t, kDedicatedWorkerMessageTarget);

      await create_broadcast_channel(
          t, broadcast_channel_name, dedicated_worker, dedicated_worker);

      const handles = await create_file_system_handles(root);

      broadcast_channel.postMessage(
          {type: 'receive-file-system-handles', cloned_handles: handles});

      const expected_response_count = 3;
      const responses = [];
      for (let i = 0; i < expected_response_count; ++i) {
        const message_event =
            await broadcast_channel_event_watcher.wait_for('message');
        responses.push(message_event.data);
      }

      const expected_serialized_handles = await serialize_handles(handles);

      for (let i = 0; i < responses.length; ++i) {
        assert_equals(
            responses[i].type, 'receive-serialized-file-system-handles',
            'The test runner must receive a "serialized-file-system-handles" ' +
                `message response. Actual response: ${responses[i]}`);

        assert_equals_serialized_handles(
            responses[i].serialized_handles, expected_serialized_handles);

        await assert_equals_cloned_handles(
            responses[i].cloned_handles, handles);
      }
    },
    'Send and receive messages using a broadcast channel in an iframe, ' +
        'dedicated worker and service worker.');
