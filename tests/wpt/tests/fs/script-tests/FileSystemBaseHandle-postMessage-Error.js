'use strict';

// This script depends on the following scripts:
//    /fs/resources/messaging-helpers.js
//    /fs/resources/messaging-blob-helpers.js
//    /fs/resources/messaging-serialize-helpers.js
//    /fs/resources/test-helpers.js
//    /common/get-host-info.sub.js
//    /service-workers/service-worker/resources/test-helpers.sub.js

// Define URL constants for cross origin windows.
const kRemoteOrigin = get_host_info().HTTPS_REMOTE_ORIGIN;
const kRemoteOriginDocumentMessageTarget = `${kRemoteOrigin}${base_path()}` +
  kDocumentMessageTarget;

// Sending a FileSystemHandle to a cross origin |target| through postMessage()
// must dispatch the 'messageerror' event.
//
// This test sends a FileSystemHandle to |target|. |target| responds with a
// serialized MessageEvent from the 'messageerror' event, allowing the test
// runner to verify MessageEvent properties.
async function do_send_message_error_test(
  test,
  root_dir,
  receiver,
  target,
  target_origin,
  // False when the MessageEvent's source is null.
  expected_has_source,
  // The origin of MessageEvents received by |target|.
  expected_origin) {
  const message_watcher = new EventWatcher(test, receiver, 'message');

  // Send a file to |target|.
  const file = await createFileWithContents(
      'test-error-file', 'test-error-file-contents', root_dir);
  target.postMessage(
    { type: 'receive-file-system-handles', cloned_file_system_handles: [file] },
    { targetOrigin: target_origin });

  // Wait for |target| to respond with results.
  let message_event = await message_watcher.wait_for('message');
  const first_response = message_event.data;
  assert_equals(first_response.type, 'serialized-message-error',
    'The test runner must receive a "serialized-message-error" message ' +
    'in response to a FileSystemFileHandle message.');

  // Verify the results.
  assert_equals_serialized_message_error_event(
    first_response.serialized_message_error_event,
    expected_origin, expected_has_source);

  // Send a directory to |target|.
  const directory = await createDirectory('test-error-directory', root_dir);

  target.postMessage(
    {
      type: 'receive-file-system-handles',
      cloned_file_system_handles: [directory]
    }, { targetOrigin: target_origin });

  // Wait for |target| to respond with results.
  message_event = await message_watcher.wait_for('message');
  const second_response = message_event.data;
  assert_equals(second_response.type, 'serialized-message-error',
    'The test runner must receive a "serialized-message-error" message ' +
    'response to a FileSystemDirectoryHandle message.');

  // Verify the results.
  assert_equals_serialized_message_error_event(
    second_response.serialized_message_error_event,
    expected_origin, expected_has_source);
}

// This test receives a FileSystemHandle from |target|. This test runner
// must dispatch the 'messageerror' event after receiving a handle from target.
async function do_receive_message_error_test(
  test,
  receiver,
  target,
  target_origin,
  // False when the MessageEvent's source is null.
  expected_has_source,
  // The origin of MessageEvents received by this test runner.
  expected_origin) {
  const error_watcher = new EventWatcher(test, receiver, 'messageerror');

  // Receive a file from |target|.
  target.postMessage(
    { type: 'create-file' }, { targetOrigin: target_origin });
  const first_error = await error_watcher.wait_for('messageerror');
  const serialized_first_error = serialize_message_error_event(first_error);
  assert_equals_serialized_message_error_event(
    serialized_first_error, expected_origin, expected_has_source);

  // Receive a directory from |target|.
  target.postMessage(
    { type: 'create-directory' }, { targetOrigin: target_origin });
  const second_error = await error_watcher.wait_for('messageerror');
  const serialized_second_error = serialize_message_error_event(second_error);
  assert_equals_serialized_message_error_event(
    serialized_second_error, expected_origin, expected_has_source);
}

// Performs the send message error test followed by the receive message error
// test.
async function do_send_and_receive_message_error_test(
  test,
  root_dir,
  receiver,
  target,
  target_origin,
  // False when the MessageEvent's source is null.
  expected_has_source,
  // The origin of MessageEvents received by |target|.
  expected_origin,
  // The origin of MessageEvents received by this test runner.
  expected_remote_origin) {
  await do_send_message_error_test(
    test, root_dir, receiver, target, target_origin, expected_has_source,
    expected_origin);
  await do_receive_message_error_test(
    test, receiver, target, target_origin, expected_has_source,
    expected_remote_origin);
}

// Runs the same test as do_send_message_error_test(), but uses a MessagePort.
// This test starts by establishing a message channel between the test runner
// and |target|.
async function do_send_message_port_error_test(
  test, root_dir, target, target_origin) {
  const message_port = create_message_channel(target, target_origin);
  await do_send_message_error_test(
    test, root_dir, /*receiver=*/message_port, /*target=*/message_port,
    /*target_origin=*/undefined, /*expected_has_source=*/false,
    /*expected_origin=*/'', /*expected_remote_origin=*/'');
}

// Runs the same test as do_receive_message_error_test(), but uses a MessagePort.
async function do_receive_message_port_error_test(
  test, target, target_origin) {
  const message_port = create_message_channel(target, target_origin);
  await do_receive_message_error_test(
    test, /*receiver=*/message_port, /*target=*/message_port,
    /*target_origin=*/undefined, /*expected_has_source=*/false,
    /*expected_origin=*/'');
}

// Runs the same test as do_send_and_receive_message_error_test(), but uses a
// MessagePort.
async function do_send_and_receive_message_port_error_test(
  test, root_dir, target, target_origin) {
  await do_send_message_port_error_test(
    test, root_dir, target, target_origin);
  await do_receive_message_port_error_test(
    test, target, target_origin);
}

directory_test(async (t, root_dir) => {
  const iframe = await add_iframe(
    t, { src: kRemoteOriginDocumentMessageTarget });
  await do_send_and_receive_message_error_test(
    t, root_dir, /*receiver=*/self, /*target=*/iframe.contentWindow,
    /*target_origin=*/'*', /*expected_has_source=*/true,
    /*expected_origin=*/location.origin,
    /*expected_remote_origin=*/kRemoteOrigin);
}, 'Fail to send and receive messages using a cross origin iframe.');

directory_test(async (t, root_dir) => {
  const iframe = await add_iframe(t, { src: kRemoteOriginDocumentMessageTarget });
  await do_send_and_receive_message_port_error_test(
    t, root_dir, /*target=*/iframe.contentWindow, /*target_origin=*/'*');
}, 'Fail to send and receive messages using a cross origin message port in ' +
'an iframe.');

directory_test(async (t, root_dir) => {
  const iframe = await add_iframe(
    t, { src: kDocumentMessageTarget, sandbox: 'allow-scripts' });

  await do_send_message_error_test(
    t, root_dir, /*receiver=*/self, /*target=*/iframe.contentWindow,
    /*target_origin=*/'*', /*expected_has_source*/true,
    /*expected_origin=*/location.origin);
}, 'Fail to send to a sandboxed iframe.');

directory_test(async (t, root_dir) => {
  const iframe = await add_iframe(
    t, { src: kDocumentMessageTarget, sandbox: 'allow-scripts' });
  await do_send_message_port_error_test(
    t, root_dir, /*target=*/iframe.contentWindow, /*target_origin=*/'*');
}, 'Fail to send messages using a message port to a sandboxed ' +
'iframe.');

directory_test(async (t, root_dir) => {
  const iframe_data_uri = await create_message_target_data_uri(t);
  const iframe = await add_iframe(t, { src: iframe_data_uri });
  await do_send_message_error_test(t, root_dir, /*receiver=*/self,
    /*target=*/iframe.contentWindow, /*target_origin=*/'*',
    /*expected_has_source*/true, /*expected_origin=*/location.origin);
  // Do not test receiving FileSystemHandles from the data URI iframe. Data URI
  // iframes are insecure and do not expose the File System APIs.
}, 'Fail to send messages to a data URI iframe.');

directory_test(async (t, root_dir) => {
  const iframe_data_uri = await create_message_target_data_uri(t);
  const iframe = await add_iframe(t, { src: iframe_data_uri });
  await do_send_message_port_error_test(
    t, root_dir, /*target=*/iframe.contentWindow, /*target_origin=*/'*');
}, 'Fail to send messages using a message port in a data URI iframe.');

directory_test(async (t, root_dir) => {
  const child_window = await open_window(t, kRemoteOriginDocumentMessageTarget);
  await do_send_and_receive_message_error_test(
    t, root_dir, /*receiver=*/self, /*target=*/child_window, /*target_origin=*/'*',
    /*expected_has_source=*/true, /*expected_origin=*/location.origin,
    /*expected_remote_origin=*/kRemoteOrigin);
}, 'Fail to send and receive messages using a cross origin window.');

directory_test(async (t, root_dir) => {
  const child_window = await open_window(t, kRemoteOriginDocumentMessageTarget);
  await do_send_message_port_error_test(
    t, root_dir, /*target=*/child_window, /*target_origin=*/'*');
}, 'Fail to send and receive messages using a cross origin message port in ' +
'a window.');

directory_test(async (t, root_dir) => {
  const url = `${kDocumentMessageTarget}?pipe=header(Content-Security-Policy` +
    ', sandbox allow-scripts)';
  const child_window = await open_window(t, url);
  await do_send_message_error_test(
    t, root_dir, /*receiver=*/self, /*target=*/child_window,
    /*target_origin=*/'*', /*expected_has_source*/true,
    /*expected_origin=*/location.origin);
}, 'Fail to send messages to  a sandboxed window.');

directory_test(async (t, root_dir) => {
  const url = `${kDocumentMessageTarget}?pipe=header(Content-Security-Policy` +
    ', sandbox allow-scripts)';
  const child_window = await open_window(t, url);
  await do_send_message_port_error_test(
    t, root_dir, /*target=*/child_window, /*target_origin=*/'*');
}, 'Fail to send messages using a message port to a sandboxed ' +
'window.');
