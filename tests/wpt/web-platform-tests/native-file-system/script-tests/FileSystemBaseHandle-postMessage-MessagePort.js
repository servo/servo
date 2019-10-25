'use strict';

// This script depends on the following scripts:
//    /native-file-system/resources/messaging-helpers.js
//    /native-file-system/resources/messaging-blob-helpers.js
//    /native-file-system/resources/messaging-serialize-helpers.js
//    /native-file-system/resources/test-helpers.js
//    /service-workers/service-worker/resources/test-helpers.sub.js

// Runs the same test as do_post_message_test(), but uses a MessagePort.
// This test starts by establishing a message channel between the test runner
// and |target|. Afterwards, the test sends FileSystemHandles through the
// message port channel.
async function do_message_port_test(test, root_dir, target, target_origin) {
  const message_port = create_message_channel(target, target_origin);
  await do_post_message_test(
    test, root_dir, /*receiver=*/message_port, /*target=*/message_port);
}

directory_test(async (t, root_dir) => {
  const iframe = await add_iframe(t, { src: kDocumentMessageTarget });
  await do_message_port_test(
    t, root_dir, /*target=*/iframe.contentWindow, /*target_origin=*/'*');
}, 'Send and receive messages using a message port in a same origin ' +
'iframe.');

directory_test(async (t, root_dir) => {
  const iframe = await add_iframe(t, {
    src: kDocumentMessageTarget,
    sandbox: 'allow-scripts allow-same-origin'
  });
  await do_message_port_test(
    t, root_dir, /*target=*/iframe.contentWindow, /*target_origin=*/'*');
}, 'Send and receive messages using a message port in a sandboxed same ' +
'origin iframe.');

directory_test(async (t, root_dir) => {
  const blob_url = await create_message_target_blob_url(t);
  const iframe = await add_iframe(t, { src: blob_url });
  await do_message_port_test(
    t, root_dir, /*target=*/iframe.contentWindow, /*target_origin=*/'*');
}, 'Send and receive messages using a message port in a blob iframe.');

directory_test(async (t, root_dir) => {
  const iframe_html = await create_message_target_html_without_subresources(t);
  const iframe = await add_iframe(t, { srcdoc: iframe_html });
  await do_message_port_test(
    t, root_dir, /*target=*/iframe.contentWindow, /*target_origin=*/'*');
}, 'Send and receive messages using a message port in an iframe srcdoc.');

directory_test(async (t, root_dir) => {
  const child_window = await open_window(t, kDocumentMessageTarget);
  await do_message_port_test(
    t, root_dir, /*target=*/child_window, /*target_origin=*/'*');
}, 'Send and receive messages using a message port in a same origin ' +
'window.');

directory_test(async (t, root_dir) => {
  const blob_url = await create_message_target_blob_url(t);
  const child_window = await open_window(t, blob_url);
  await do_message_port_test(
    t, root_dir, /*target=*/child_window, /*target_origin=*/'*');
}, 'Send and receive messages using a message port in a blob window.');

directory_test(async (t, root_dir) => {
  const url = `${kDocumentMessageTarget}?pipe=header(Content-Security-Policy` +
    ', sandbox allow-scripts allow-same-origin)';
  const child_window = await open_window(t, url);
  await do_message_port_test(
    t, root_dir, /*target=*/child_window, /*target_origin=*/'*');
}, 'Send and receive messages using a message port in a sandboxed same ' +
'origin window.');

directory_test(async (t, root_dir) => {
  const dedicated_worker =
    create_dedicated_worker(t, kDedicatedWorkerMessageTarget);
  await do_message_port_test(t, root_dir, /*target=*/dedicated_worker);
}, 'Send and receive messages using a message port in a dedicated ' +
'worker.');

directory_test(async (t, root_dir) => {
  const scope = `${kServiceWorkerMessageTarget}` +
    '?post-message-to-message-port-with-file-handle';
  const registration = await create_service_worker(
    t, kServiceWorkerMessageTarget, scope);
  await do_message_port_test(t, root_dir, /*target=*/registration.installing);
}, 'Send and receive messages using a message port in a service ' +
'worker.');

if (self.SharedWorker !== undefined) {
  directory_test(async (t, root_dir) => {
    const shared_worker = new SharedWorker(kSharedWorkerMessageTarget);
    shared_worker.port.start();
    await do_message_port_test(t, root_dir, /*target=*/shared_worker.port);
  }, 'Send and receive messages using a message port in a shared ' +
  ' worker.');
}
