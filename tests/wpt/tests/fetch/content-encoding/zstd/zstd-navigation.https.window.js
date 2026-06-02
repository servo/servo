// META: script=../../../service-workers/service-worker/resources/test-helpers.sub.js

promise_test(async t => {
  const frame = await with_iframe('./resources/hello.html.zst');
  assert_equals(frame.contentDocument.body.textContent, 'Hello world');
}, `Naigation to zstd encoded page`);
