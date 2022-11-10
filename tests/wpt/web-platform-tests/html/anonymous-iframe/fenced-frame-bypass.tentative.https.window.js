// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/html/cross-origin-embedder-policy/credentialless/resources/common.js
// META: script=./resources/common.js
// META: timeout=long

setup(() => {
  assert_implements(window.HTMLFencedFrameElement,
    "HTMLFencedFrameElement is not supported.");
})

// 4 actors:
//                         A (this document)
//   ┌─────────────────────┴──┐
// ┌─┼───────────────────┐    D  (anonymous-iframe)
// │ B (fenced-frame)    │
// │ │                   │
// │ C (anonymous-iframe)│
// └─────────────────────┘
//
// This test whether the two anonymous iframe can communicate and bypass the
// fencedframe boundary. This shouldn't happen.
promise_test(async test => {
  const cross_origin = get_host_info().HTTPS_REMOTE_ORIGIN;
  const msg_queue = token();

  // Create the the 3 actors.
  const anonymous_iframe_1 = newAnonymousIframe(cross_origin);
  const fenced_frame = newFencedFrame(cross_origin);
  send(fenced_frame, `
    const importScript = ${importScript};
    await importScript("/common/utils.js");
    await importScript("/html/cross-origin-embedder-policy/credentialless" +
      "/resources/common.js");
    await importScript("/html/anonymous-iframe/resources/common.js");
    const support_loading_mode_fenced_frame =
      "|header(Supports-Loading-Mode,fenced-frame)";
    const anonymous_iframe_2 =
      newAnonymousIframe("${cross_origin}", support_loading_mode_fenced_frame);
    send("${msg_queue}", anonymous_iframe_2);
  `);
  const anonymous_iframe_2 = await receive(msg_queue);

  // Try to communicate using BroadCastChannel, in between the two
  // AnonymousIframe.
  const bc_key = token();
  send(anonymous_iframe_1, `
    const bc = new BroadcastChannel("${bc_key}");
    bc.onmessage = event => send("${msg_queue}", event.data);
    send("${msg_queue}", "BroadcastChannel registered");
  `);
  assert_equals(await receive(msg_queue), "BroadcastChannel registered");
  await send(anonymous_iframe_2, `
    const bc = new BroadcastChannel("${bc_key}");
    bc.postMessage("Can communicate");
  `);
  test.step_timeout(() => {
    send(msg_queue, "Cannot communicate");
  }, 4000);

  assert_equals(await receive(msg_queue), "Cannot communicate");
})
