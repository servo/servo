// META: title=Cross-origin navigation started from unload handler must be ignored
// META: script=../resources/helpers.js

promise_test(async () => {
  const iframe = await addIframe();

  iframe.contentWindow.addEventListener("unload", () => {
    iframe.contentWindow.location.href = "//{{hosts[][www]}}/common/blank.html?fail";
  });

  iframe.src = "/common/blank.html?pass";

  await waitForIframeLoad(iframe);
  assert_equals(iframe.contentWindow.location.search, "?pass");
});
