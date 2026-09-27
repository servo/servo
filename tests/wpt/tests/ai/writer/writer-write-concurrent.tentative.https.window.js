// META: title=Writer Write Concurrent
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const writer = await createWriter();
  await Promise.all([writer.write(kTestPrompt), writer.write(kTestPrompt)]);
}, 'Multiple Writer.write() calls with identical inputs are resolved successfully');

promise_test(async () => {
  const writer = await createWriter();
  await Promise.all([writer.write(kTestPrompt), writer.write(kTestPrompt2)]);
}, 'Multiple Writer.write() calls with divergent inputs are resolved successfully');
