// META: title=Writer
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  assert_true(!!Writer);
}, 'Writer must be defined.');

promise_test(async t => {
  // Creating Writer without user activation rejects with NotAllowedError.
  await promise_rejects_dom(t, 'NotAllowedError', Writer.create());

  // Creating Writer with user activation succeeds.
  await createWriter();

  // Expect available after create.
  assert_equals(await Writer.availability(), 'available');

  // Now that it is available, we should no longer need user activation.
  await Writer.create();
}, 'Writer.create() requires user activation when availability is "downloadable."');

promise_test(async () => {
  const writer = await createWriter();
  assert_equals(Object.prototype.toString.call(writer), '[object Writer]');
}, 'Writer.create() must be return a Writer.');

promise_test(async () => {
  await testMonitor(createWriter);
}, 'Writer.create() notifies its monitor on downloadprogress');

promise_test(async t => {
  await testCreateMonitorWithAbort(t, Writer.create);
}, 'Progress events are not emitted after aborted.');

promise_test(async () => {
  const writer = await createWriter();
  assert_equals(writer.sharedContext, '');
  assert_equals(writer.tone, 'neutral');
  assert_equals(writer.format, 'plain-text');
  assert_equals(writer.length, 'medium');
}, 'Writer.create() default values.');

promise_test(async (t) => {
  const controller = new AbortController();
  controller.abort();
  const createPromise = createWriter({signal: controller.signal});
  await promise_rejects_dom(t, 'AbortError', createPromise);
}, 'Writer.create() call with an aborted signal.');

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
}, 'Creating a Writer with expectedInputLanguages');


promise_test(async (t) => {
  promise_rejects_js(
      t, RangeError,
      createWriter({expectedInputLanguages: ['en-abc-invalid']}));
}, 'Creating a Writer with malformed language string');

promise_test(async () => {
  const writer = await createWriter({expectedContextLanguages: ['en']});
  assert_array_equals(writer.expectedContextLanguages, ['en']);
}, 'Creating a Writer with expectedContextLanguages');

promise_test(async () => {
  const writer = await createWriter({outputLanguage: 'en'});
  assert_equals(writer.outputLanguage, 'en');
}, 'Creating a Writer with outputLanguage');

promise_test(async () => {
  const writer = await createWriter({});
  assert_equals(writer.expectedInputLanguages, null);
  assert_equals(writer.expectedContextLanguages, null);
  assert_equals(writer.outputLanguage, null);
}, 'Creating a Writer without optional attributes');

promise_test(async (t) => {
  const writer = await createWriter();
  let result = await writer.write('');
  assert_equals(result, '');
  result = await writer.write(' ');
  assert_equals(result, '');
}, 'Writer.write() with an empty input or whitespace returns an empty text');

promise_test(async (t) => {
  const writer = await createWriter();
  const result = await writer.write('hello', {context: ' '});
  assert_not_equals(result, '');
}, 'Writer.write() with a whitespace context returns a non-empty result');

promise_test(async (t) => {
  const writer = await createWriter();
  writer.destroy();
  await promise_rejects_dom(t, 'InvalidStateError', writer.write('hello'));
}, 'Writer.write() fails after destroyed');

promise_test(async (t) => {
  const writer = await createWriter();
  writer.destroy();
  assert_throws_dom('InvalidStateError', () => writer.writeStreaming('hello'));
}, 'Writer.writeStreaming() fails after destroyed');

promise_test(async () => {
  const writer = await createWriter();
  const result = await writer.measureInputUsage(kTestPrompt);
  assert_greater_than(result, 0);
}, 'Writer.measureInputUsage() returns non-empty result');

promise_test(async () => {
  const writer = await createWriter();
  const result = await writer.write(kTestPrompt, {context: kTestContext});
  assert_equals(typeof result, 'string');
}, 'Simple Writer.write() call');

promise_test(async () => {
  const writer = await createWriter();
  const streamingResponse =
      writer.writeStreaming(kTestPrompt, {context: kTestContext});
  assert_equals(
      Object.prototype.toString.call(streamingResponse),
      '[object ReadableStream]');
  let result = '';
  for await (const chunk of streamingResponse) {
    result += chunk;
  }
  assert_greater_than(result.length, 0);
}, 'Simple Writer.writeStreaming() call');

promise_test(async () => {
  const writer = await createWriter();
  await Promise.all([writer.write(kTestPrompt), writer.write(kTestPrompt)]);
}, 'Multiple Writer.write() calls are resolved successfully.');

promise_test(async () => {
  const writer = await createWriter();
  await Promise.all(
      [writer.writeStreaming(kTestPrompt), writer.writeStreaming(kTestPrompt)]);
}, 'Multiple Writer.writeStreaming() calls are resolved successfully.');
