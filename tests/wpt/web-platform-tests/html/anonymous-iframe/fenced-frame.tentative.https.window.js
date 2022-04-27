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

// Check whether this anonymous bit propagates toward FencedFrame. It shouldn't.
promise_test(async test => {
  const origin = get_host_info().HTTPS_ORIGIN;
  const msg_queue = token();

  // 1. Create an anonymous iframe.
  const frame_anonymous = newAnonymousIframe(origin);

  // 2. Create a FencedFrame within it.
  send(frame_anonymous, `
    const importScript = ${importScript};
    await importScript("/common/utils.js");
    await importScript("/html/cross-origin-embedder-policy/credentialless" +
      "/resources/common.js");
    await importScript("/html/anonymous-iframe/resources/common.js");
    const frame_fenced = newFencedFrame("${origin}");
    send("${msg_queue}", frame_fenced);
  `);
  const frame_fenced = await receive(msg_queue);

  // 3. Expect it not to be considered anonymous.
  send(frame_fenced, `
    send("${msg_queue}", window.anonymous);
  `);
  assert_equals(await receive(msg_queue), "false",
    "Check window.anonymous in FencedFrame");
}, 'FencedFrame within an AnonymousIframe is not anonymous')
