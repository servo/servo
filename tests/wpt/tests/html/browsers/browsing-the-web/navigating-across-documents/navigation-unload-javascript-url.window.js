// META: title=javascript: URL navigation started from unload handler must be ignored
// META: script=../resources/helpers.js

promise_test(async () => {
  const iframe = await addIframe();

  iframe.contentWindow.addEventListener("unload", () => {
    iframe.contentWindow.location.href =
      `javascript:"unload<script>parent.postMessage('fail', '*');</script>"`;
  });

  iframe.src =
    `javascript:"load<script>parent.postMessage('pass', '*')</script>"`;
  assert_equals(await waitForMessage(iframe.contentWindow), "pass");
});
