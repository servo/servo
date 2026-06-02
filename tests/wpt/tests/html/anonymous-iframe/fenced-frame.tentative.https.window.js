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

// Check whether this credentialless bit propagates toward FencedFrame. It
// shouldn't.
promise_test(async test => {
  const origin = get_host_info().HTTPS_ORIGIN;
  const msg_queue = token();

  // 1. Create a credentialless iframe.
  const iframe_credentialless = newIframeCredentialless(origin);

  // 2. Create a FencedFrame within it.
  send(iframe_credentialless, `
    const importScript = ${importScript};
    await importScript("/common/utils.js");
    await importScript("/fenced-frame/resources/utils.js");
    await importScript("/html/cross-origin-embedder-policy/credentialless" +
      "/resources/common.js");
    await importScript("/html/anonymous-iframe/resources/common.js");
    const frame_fenced = await newFencedFrame("${origin}");
    send("${msg_queue}", frame_fenced);
  `);
  // TODO: Properly generate a fenced frame to check credentialless.
  assert_true(false, "Fenced frame cannot be created.");
  const frame_fenced = await receive(msg_queue);

  // 3. Expect it not to be considered credentialless.
  send(frame_fenced, `
    send("${msg_queue}", window.credentialless);
  `);
  // TODO: Properly generate a fenced frame which can perform this check.
  assert_equals(await receive(msg_queue), "false",
    "Check window.credentialless in FencedFrame");
}, 'FencedFrame within a credentialless iframe is not credentialless')
