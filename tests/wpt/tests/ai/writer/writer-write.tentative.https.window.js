// META: title=Writer Write
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async (t) => {
  const writer = await createWriter();
  let result = await writer.write('');
  assert_equals(result, '');
}, 'Writer.write() with an empty input returns an empty text');

promise_test(async (t) => {
  const writer = await createWriter();
  let result = await writer.write(' ');
  assert_equals(result, '');
}, 'Writer.write() with a whitespace input returns an empty text');

promise_test(async (t) => {
  const writer = await createWriter();
  const result = await writer.write(kTestPrompt, {context: ' '});
  assert_not_equals(result, '');
}, 'Writer.write() with a whitespace context returns a non-empty result');

promise_test(async t => {
  await testDestroy(t, createWriter, {}, [
    writer => writer.write(kTestPrompt),
    writer => writer.measureInputUsage(kTestPrompt),
  ]);
}, 'Calling Writer.destroy() aborts calls to write and measureInputUsage.');

promise_test(async t => {
  await testCreateAbort(t, createWriter, {}, [
    writer => writer.write(kTestPrompt),
    writer => writer.measureInputUsage(kTestPrompt),
  ]);
}, 'Writer.create()\'s abort signal destroys its Writer after creation.');

promise_test(async () => {
  const writer = await createWriter();
  const result = await writer.write(kTestPrompt, {context: kTestContext});
  assert_equals(typeof result, 'string');
}, 'Simple Writer.write() call');

promise_test(async () => {
  const writer = await createWriter();
  await Promise.all([writer.write(kTestPrompt), writer.write(kTestPrompt)]);
}, 'Multiple Writer.write() calls are resolved successfully');
