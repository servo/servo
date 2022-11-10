// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/html/cross-origin-embedder-policy/credentialless/resources/common.js
// META: script=./resources/common.js

// This test verifies the behavior of the initial empty document nested inside
// anonymous iframes.
//
// The following tree of frames and documents is used:
//  A
//  ├──B (anonymous)
//  │  └──D (initial empty document)
//  └──C (control)
//     └──E (initial empty document)
//
// Storage used for D and E must be different.
promise_test(async test => {
  const iframe_B = newAnonymousIframe(origin);
  const iframe_C = newIframe(origin);

  // Create iframe_D and store a value in localStorage.
  const key_D = token();
  const value_D = "value_D";
  const queue_B = token();
  send(iframe_B, `
    const iframe_D = document.createElement("iframe");
    document.body.appendChild(iframe_D);
    iframe_D.contentWindow.localStorage.setItem("${key_D}","${value_D}");
    send("${queue_B}", "Done");
  `);

  // Create iframe_E and store a value in localStorage.
  const key_E = token();
  const value_E = "value_E";
  const queue_C = token();
  send(iframe_C, `
    const iframe_E = document.createElement("iframe");
    document.body.appendChild(iframe_E);
    iframe_E.contentWindow.localStorage.setItem("${key_E}","${value_E}");
    send("${queue_C}", "Done");
  `);

  assert_equals(await receive(queue_B), "Done");
  assert_equals(await receive(queue_C), "Done");

  // Try to load both values from both contexts:
  send(iframe_B, `
    const iframe_D = document.querySelector("iframe");
    const value_D = iframe_D.contentWindow.localStorage.getItem("${key_D}");
    const value_E = iframe_D.contentWindow.localStorage.getItem("${key_E}");
    send("${queue_B}", value_D);
    send("${queue_B}", value_E);
  `);
  send(iframe_C, `
    const iframe_E = document.querySelector("iframe");
    const value_D = iframe_E.contentWindow.localStorage.getItem("${key_D}");
    const value_E = iframe_E.contentWindow.localStorage.getItem("${key_E}");
    send("${queue_C}", value_D);
    send("${queue_C}", value_E);
  `);

  // Verify the anonymous iframe and the normal one do not have access to each
  // other.
  assert_equals(await receive(queue_B), value_D); // key_D
  assert_equals(await receive(queue_B), "");      // key_E
  assert_equals(await receive(queue_C), "");      // key_D
  assert_equals(await receive(queue_C), value_E); // key_E
}, "Local storage is correctly partitioned with regards to anonymous iframe " +
   "in initial empty documents.");
