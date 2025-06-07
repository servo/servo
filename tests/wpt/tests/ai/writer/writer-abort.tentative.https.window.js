// META: title=Writer Abort
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await testAbortPromise(t, signal => {
    return createWriter({signal: signal});
  });
}, 'Aborting Writer.create()');

promise_test(async t => {
  const writer = await createWriter();
  await testAbortPromise(t, signal => {
    return writer.write(kTestPrompt, { signal: signal });
  });
}, 'Aborting Writer.write()');

promise_test(async t => {
  const writer = await createWriter();
  await testAbortReadableStream(t, signal => {
    return writer.writeStreaming(kTestPrompt, { signal: signal });
  });
}, 'Aborting Writer.writeStreaming()');

promise_test(async (t) => {
  const writer = await createWriter();
  const controller = new AbortController();
  const streamingResponse = writer.writeStreaming(
    kTestPrompt, { signal: controller.signal });
  for await (const chunk of streamingResponse);  // Do nothing
  controller.abort();
}, 'Aborting Writer.writeStreaming() after finished reading');
