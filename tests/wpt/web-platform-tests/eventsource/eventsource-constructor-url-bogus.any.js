// META: global=window,worker
// META: title=EventSource: constructor (invalid URL)
// META: script=/resources/idlharness.js

test(() => {
  assert_throws('SyntaxError', () => { new EventSource("http://this is invalid/"); });
});

done();