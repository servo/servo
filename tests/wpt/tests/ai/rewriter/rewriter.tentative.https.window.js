// META: title=Rewriter
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  assert_true(!!Rewriter);
}, 'Rewriter must be defined.');

promise_test(async t => {
  // Creating Rewriter without user activation rejects with NotAllowedError.
  await promise_rejects_dom(t, 'NotAllowedError', Rewriter.create());

  // Creating Rewriter with user activation succeeds.
  await createRewriter();

  // Expect available after create.
  assert_equals(await Rewriter.availability(), 'available');

  // Now that it is available, we should no longer need user activation.
  await Rewriter.create();
}, 'Rewriter.create() requires user activation when availability is "downloadable."');

promise_test(async () => {
  const rewriter = await createRewriter();
  assert_equals(Object.prototype.toString.call(rewriter), '[object Rewriter]');
}, 'Rewriter.create() must be return a Rewriter.');

promise_test(async () => {
  await testMonitor(createRewriter);
}, 'Rewriter.create() notifies its monitor on downloadprogress');

promise_test(async t => {
  await testCreateMonitorWithAbort(t, Rewriter.create);
}, 'Progress events are not emitted after aborted.');

promise_test(async () => {
  const rewriter = await createRewriter();
  assert_equals(rewriter.sharedContext, '');
  assert_equals(rewriter.tone, 'as-is');
  assert_equals(rewriter.format, 'as-is');
  assert_equals(rewriter.length, 'as-is');
}, 'Rewriter.create() default values.');

promise_test(async () => {
  const sharedContext = 'This is a shared context string';
  const rewriter = await createRewriter({sharedContext: sharedContext});
  assert_equals(rewriter.sharedContext, sharedContext);
}, 'Rewriter.sharedContext');

promise_test(async () => {
  const rewriter = await createRewriter({tone: 'more-formal'});
  assert_equals(rewriter.tone, 'more-formal');
}, 'Creating a Rewriter with "more-formal" tone');

promise_test(async () => {
  const rewriter = await createRewriter({tone: 'more-casual'});
  assert_equals(rewriter.tone, 'more-casual');
}, 'Creating a Rewriter with "more-casual" tone');

promise_test(async () => {
  const rewriter = await createRewriter({format: 'plain-text'});
  assert_equals(rewriter.format, 'plain-text');
}, 'Creating a Rewriter with "plain-text" format');

promise_test(async () => {
  const rewriter = await createRewriter({format: 'markdown'});
  assert_equals(rewriter.format, 'markdown');
}, 'Creating a Rewriter with "markdown" format');

promise_test(async () => {
  const rewriter = await createRewriter({length: 'shorter'});
  assert_equals(rewriter.length, 'shorter');
}, 'Creating a Rewriter with "shorter" length');

promise_test(async () => {
  const rewriter = await createRewriter({length: 'longer'});
  assert_equals(rewriter.length, 'longer');
}, 'Creating a Rewriter with "longer" length');

promise_test(async () => {
  const rewriter = await createRewriter({expectedInputLanguages: ['en']});
  assert_array_equals(rewriter.expectedInputLanguages, ['en']);
}, 'Creating a Rewriter with expectedInputLanguages');

promise_test(async () => {
  const rewriter = await createRewriter({expectedContextLanguages: ['en']});
  assert_array_equals(rewriter.expectedContextLanguages, ['en']);
}, 'Creating a Rewriter with expectedContextLanguages');

promise_test(async () => {
  const rewriter = await createRewriter({outputLanguage: 'en'});
  assert_equals(rewriter.outputLanguage, 'en');
}, 'Creating a Rewriter with outputLanguage');

promise_test(async () => {
  const rewriter = await createRewriter({});
  assert_equals(rewriter.expectedInputLanguages, null);
  assert_equals(rewriter.expectedContextLanguages, null);
  assert_equals(rewriter.outputLanguage, null);
}, 'Creating a Rewriter without optional attributes');

promise_test(
    async (t) => {
      const rewriter = await createRewriter();
      let result = await rewriter.rewrite('');
      assert_equals(result, '');
      result = await rewriter.rewrite(' ');
      assert_equals(result, ' ');
    },
    'Rewriter.rewrite() with an empty input or whitespace returns the ' +
        'original input');

promise_test(async (t) => {
  const rewriter = await createRewriter();
  const result = await rewriter.rewrite('hello', {context: ' '});
  assert_not_equals(result, '');
}, 'Rewriter.rewrite() with a whitespace context returns a non-empty result');

promise_test(async (t) => {
  const rewriter = await createRewriter();
  rewriter.destroy();
  await promise_rejects_dom(t, 'InvalidStateError', rewriter.rewrite('hello'));
}, 'Rewriter.rewrite() fails after destroyed');

promise_test(async (t) => {
  const rewriter = await createRewriter();
  rewriter.destroy();
  assert_throws_dom(
      'InvalidStateError', () => rewriter.rewriteStreaming('hello'));
}, 'Rewriter.rewriteStreaming() fails after destroyed');

promise_test(async () => {
  const rewriter = await createRewriter();
  const result = await rewriter.measureInputUsage(kTestPrompt);
  assert_greater_than(result, 0);
}, 'Rewriter.measureInputUsage() returns non-empty result');

promise_test(async () => {
  const rewriter = await createRewriter();
  const result = await rewriter.rewrite(kTestPrompt, {context: kTestContext});
  assert_equals(typeof result, 'string');
}, 'Simple Rewriter.rewrite() call');

promise_test(async () => {
  const rewriter = await createRewriter();
  const streamingResponse =
      rewriter.rewriteStreaming(kTestPrompt, {context: kTestContext});
  assert_equals(
      Object.prototype.toString.call(streamingResponse),
      '[object ReadableStream]');
  let result = '';
  for await (const chunk of streamingResponse) {
    result += chunk;
  }
  assert_greater_than(result.length, 0);
}, 'Simple Rewriter.rewriteStreaming() call');

promise_test(async () => {
  const rewriter = await createRewriter();
  await Promise.all(
      [rewriter.rewrite(kTestPrompt), rewriter.rewrite(kTestPrompt)]);
}, 'Multiple Rewriter.rewrite() calls are resolved successfully.');

promise_test(async () => {
  const rewriter = await createRewriter();
  await Promise.all([
    rewriter.rewriteStreaming(kTestPrompt),
    rewriter.rewriteStreaming(kTestPrompt)
  ]);
}, 'Multiple Rewriter.rewriteStreaming() calls are resolved successfully.');
