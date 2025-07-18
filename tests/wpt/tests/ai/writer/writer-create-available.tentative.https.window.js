// META: title=Writer Create Available
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const writer = await createWriter();
  assert_equals(typeof writer, 'object');

  assert_equals(typeof writer.write, 'function');
  assert_equals(typeof writer.writeStreaming, 'function');
  assert_equals(typeof writer.measureInputUsage, 'function');
  assert_equals(typeof writer.destroy, 'function');

  assert_equals(typeof writer.expectedContextLanguages, 'object');
  assert_equals(typeof writer.expectedInputLanguages, 'object');
  assert_equals(typeof writer.inputQuota, 'number');
  assert_equals(typeof writer.outputLanguage, 'object');
  assert_equals(typeof writer.sharedContext, 'string');

  assert_equals(typeof writer.tone, 'string');
  assert_equals(typeof writer.format, 'string');
  assert_equals(typeof writer.length, 'string');

  assert_equals(writer.tone, 'neutral');
  assert_equals(writer.format, 'plain-text');
  assert_equals(writer.length, 'medium');
}, 'Writer.create() returns a valid object with default options');

promise_test(async () => {
  await testMonitor(createWriter);
}, 'Writer.create() notifies its monitor on downloadprogress');

promise_test(async t => {
  await testCreateMonitorWithAbort(t, Writer.create);
}, 'Progress events are not emitted after aborted');

promise_test(async () => {
  const sharedContext = 'This is a shared context string';
  const writer = await createWriter({sharedContext: sharedContext});
  assert_equals(writer.sharedContext, sharedContext);
}, 'Writer.sharedContext');

promise_test(async () => {
  const writer = await createWriter({tone: 'formal'});
  assert_equals(writer.tone, 'formal');
}, 'Creating a Writer with "formal" tone');

promise_test(async () => {
  const writer = await createWriter({tone: 'casual'});
  assert_equals(writer.tone, 'casual');
}, 'Creating a Writer with "casual" tone');

promise_test(async () => {
  const writer = await createWriter({format: 'markdown'});
  assert_equals(writer.format, 'markdown');
}, 'Creating a Writer with "markdown" format');

promise_test(async () => {
  const writer = await createWriter({length: 'short'});
  assert_equals(writer.length, 'short');
}, 'Creating a Writer with "short" length');

promise_test(async () => {
  const writer = await createWriter({length: 'long'});
  assert_equals(writer.length, 'long');
}, 'Creating a Writer with "long" length');

promise_test(async () => {
  const writer = await createWriter({expectedInputLanguages: ['en']});
  assert_array_equals(writer.expectedInputLanguages, ['en']);
}, 'Writer.expectedInputLanguages');

promise_test(async () => {
  const writer = await createWriter({expectedContextLanguages: ['en']});
  assert_array_equals(writer.expectedContextLanguages, ['en']);
}, 'Writer.expectedContextLanguages');

promise_test(async () => {
  const writer = await createWriter({outputLanguage: 'en'});
  assert_equals(writer.outputLanguage, 'en');
}, 'Writer.outputLanguage');

promise_test(async (t) => {
  return promise_rejects_js(
      t, RangeError,
      createWriter({expectedInputLanguages: ['en-abc-invalid']}));
}, 'Creating Writer with malformed language string');

promise_test(async (t) => {
  let writer = await createWriter({expectedInputLanguages: ['EN']});
  assert_true(!!writer);
}, 'Writer.create() canonicalizes language tags');

promise_test(async () => {
  const writer = await createWriter({});
  assert_equals(writer.expectedInputLanguages, null);
  assert_equals(writer.expectedContextLanguages, null);
  assert_equals(writer.outputLanguage, null);
}, 'Writer optional attributes return null');
