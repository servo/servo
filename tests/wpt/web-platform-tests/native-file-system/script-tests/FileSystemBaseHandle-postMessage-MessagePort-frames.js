'use strict';

// This script depends on the following scripts:
//    /native-file-system/resources/messaging-helpers.js
//    /native-file-system/resources/messaging-blob-helpers.js
//    /native-file-system/resources/messaging-serialize-helpers.js
//    /native-file-system/resources/test-helpers.js

directory_test(
    async (t, root_dir) => {
      const iframe = await add_iframe(t, {src: kDocumentMessageTarget});
      await do_message_port_test(
          t, root_dir, /*target=*/ iframe.contentWindow,
          /*target_origin=*/ '*');
    },
    'Send and receive messages using a message port in a same origin ' +
        'iframe.');

directory_test(
    async (t, root_dir) => {
      const iframe = await add_iframe(t, {
        src: kDocumentMessageTarget,
        sandbox: 'allow-scripts allow-same-origin'
      });
      await do_message_port_test(
          t, root_dir, /*target=*/ iframe.contentWindow,
          /*target_origin=*/ '*');
    },
    'Send and receive messages using a message port in a sandboxed same ' +
        'origin iframe.');

directory_test(async (t, root_dir) => {
  const blob_url = await create_message_target_blob_url(t);
  const iframe = await add_iframe(t, {src: blob_url});
  await do_message_port_test(
      t, root_dir, /*target=*/ iframe.contentWindow, /*target_origin=*/ '*');
}, 'Send and receive messages using a message port in a blob iframe.');

directory_test(async (t, root_dir) => {
  const iframe_html = await create_message_target_html_without_subresources(t);
  const iframe = await add_iframe(t, {srcdoc: iframe_html});
  await do_message_port_test(
      t, root_dir, /*target=*/ iframe.contentWindow, /*target_origin=*/ '*');
}, 'Send and receive messages using a message port in an iframe srcdoc.');
