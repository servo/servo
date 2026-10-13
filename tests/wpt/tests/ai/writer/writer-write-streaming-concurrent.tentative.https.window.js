// META: title=Writer Write Streaming Concurrent
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const writer = await createWriter();
  const [result1, result2] = await Promise.all([
    readStream(writer.writeStreaming(kTestPrompt)),
    readStream(writer.writeStreaming(kTestPrompt)),
  ]);
  assert_greater_than(result1.length, 0);
  assert_greater_than(result2.length, 0);
}, 'Multiple Writer.writeStreaming() calls with identical inputs are resolved successfully');

promise_test(async () => {
  const writer = await createWriter();
  const [result1, result2] = await Promise.all([
    readStream(writer.writeStreaming(kTestPrompt)),
    readStream(writer.writeStreaming(kTestPrompt2)),
  ]);
  assert_greater_than(result1.length, 0);
  assert_greater_than(result2.length, 0);
}, 'Multiple Writer.writeStreaming() calls with divergent inputs are resolved successfully');
