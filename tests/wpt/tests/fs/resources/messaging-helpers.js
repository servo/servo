'use strict';

// This script depends on the following script:
//    /fs/resources/test-helpers.js
//    /service-workers/service-worker/resources/test-helpers.sub.js

// Define the URL constants used for each type of message target, including
// iframes and workers.
const kDocumentMessageTarget = '../fs/resources/message-target.html';
const kSharedWorkerMessageTarget =
    '../fs/resources/message-target-shared-worker.js';
const kServiceWorkerMessageTarget =
    '../fs/resources/message-target-service-worker.js';
const kDedicatedWorkerMessageTarget =
    '../fs/resources/message-target-dedicated-worker.js';

function create_dedicated_worker(test, url) {
  const dedicated_worker = new Worker(url);
  test.add_cleanup(() => {
    dedicated_worker.terminate();
  });
  return dedicated_worker;
}

async function create_service_worker(test, script_url, scope) {
  const registration = await service_worker_unregister_and_register(
    test, script_url, scope);
  test.add_cleanup(() => {
    return registration.unregister();
  });
  return registration;
}

// Creates an iframe and waits to receive a message from the iframe.
// Valid |options| include src, srcdoc and sandbox, which mirror the
// corresponding iframe element properties.
async function add_iframe(test, options) {
  const iframe = document.createElement('iframe');

  if (options.sandbox !== undefined) {
    iframe.sandbox = options.sandbox;
  }

  if (options.src !== undefined) {
    iframe.src = options.src;
  }

  if (options.srcdoc !== undefined) {
    iframe.srcdoc = options.srcdoc;
  }

  document.body.appendChild(iframe);
  test.add_cleanup(() => {
    iframe.remove();
  });

  await wait_for_loaded_message(self);
  return iframe;
}

// Creates a child window using window.open() and waits to receive a message
// from the child window.
async function open_window(test, url) {
  const child_window = window.open(url);
  test.add_cleanup(() => {
    child_window.close();
  });
  await wait_for_loaded_message(self);
  return child_window;
}

// Wait until |receiver| gets a message event with the data set to 'LOADED'.
// The postMessage() tests use messaging instead of the loaded event because
// cross-origin child windows from window.open() do not dispatch the loaded
// event to the parent window.
async function wait_for_loaded_message(receiver) {
  const message_promise = new Promise((resolve, reject) => {
    receiver.addEventListener('message', message_event => {
      if (message_event.data === 'LOADED') {
        resolve();
      } else {
        reject('The message target must receive a "LOADED" message response.');
      }
    });
  });
  await message_promise;
}

// Sets up a new message channel. Sends one port to |target| and then returns
// the other port.
function create_message_channel(target, target_origin) {
  const message_channel = new MessageChannel();

  const message_data =
    { type: 'receive-message-port', message_port: message_channel.port2 };
  target.postMessage(
    message_data,
    {
      transfer: [message_channel.port2],
      targetOrigin: target_origin
    });
  message_channel.port1.start();
  return message_channel.port1;
}

// Creates a variety of different FileSystemFileHandles for testing.
async function create_file_system_handles(test, root) {
  // Create some files to use with postMessage().
  const empty_file = await createEmptyFile(test, 'empty-file', root);
  const first_file = await createFileWithContents(
    test, 'first-file-with-contents', 'first-text-content', root);
  const second_file = await createFileWithContents(
    test, 'second-file-with-contents', 'second-text-content', root);

  // Create an empty directory to use with postMessage().
  const empty_directory = await createDirectory(test, 'empty-directory', root);

  // Create a directory containing both files and subdirectories to use
  // with postMessage().
  const directory_with_files =
    await createDirectory(test, 'directory-with-files', root);
  await createFileWithContents(test, 'first-file-in-directory',
    'first-directory-text-content', directory_with_files);
  await createFileWithContents(test, 'second-file-in-directory',
    'second-directory-text-content', directory_with_files);
  const subdirectory =
    await createDirectory(test, 'subdirectory', directory_with_files);
  await createFileWithContents(test, 'first-file-in-subdirectory',
    'first-subdirectory-text-content', subdirectory);

  return [
    empty_file,
    first_file,
    second_file,
    // Include the same FileSystemFileHandle twice.
    second_file,
    empty_directory,
    // Include the Same FileSystemDirectoryHandle object twice.
    empty_directory,
    directory_with_files
  ];
}

// Tests sending an array of FileSystemHandles to |target| with postMessage().
// The array includes both FileSystemFileHandles and FileSystemDirectoryHandles.
// After receiving the message, |target| accesses all cloned handles by
// serializing the properties of each handle to a JavaScript object.
//
// |target| then responds with the resulting array of serialized handles. The
// response also includes the array of cloned handles, which creates more
// clones. After receiving the response, this test runner verifies that both
// the serialized handles and the cloned handles contain the expected properties.
async function do_post_message_test(
  test, root_dir, receiver, target, target_origin) {
  // Create and send the handles to |target|.
  const handles =
    await create_file_system_handles(test, root_dir, target, target_origin);
  target.postMessage(
    { type: 'receive-file-system-handles', cloned_handles: handles },
    { targetOrigin: target_origin });

  // Wait for |target| to respond with results.
  const event_watcher = new EventWatcher(test, receiver, 'message');
  const message_event = await event_watcher.wait_for('message');
  const response = message_event.data;

  assert_equals(response.type, 'receive-serialized-file-system-handles',
    'The test runner must receive a "serialized-file-system-handles" ' +
    `message response. Actual response: ${response}`);

  // Verify the results.
  const expected_serialized_handles = await serialize_handles(handles);

  assert_equals_serialized_handles(
    response.serialized_handles, expected_serialized_handles);

  await assert_equals_cloned_handles(response.cloned_handles, handles);
}

// Runs the same test as do_post_message_test(), but uses a MessagePort.
// This test starts by establishing a message channel between the test runner
// and |target|. Afterwards, the test sends FileSystemHandles through the
// message port channel.
async function do_message_port_test(test, root_dir, target, target_origin) {
  const message_port = create_message_channel(target, target_origin);
  await do_post_message_test(
      test, root_dir, /*receiver=*/ message_port, /*target=*/ message_port);
}
