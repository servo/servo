// META: title=Rewriter Create Available
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  const rewriter = await createRewriter();
  assert_equals(typeof rewriter, 'object');

  assert_equals(typeof rewriter.rewrite, 'function');
  assert_equals(typeof rewriter.rewriteStreaming, 'function');
  assert_equals(typeof rewriter.measureInputUsage, 'function');
  assert_equals(typeof rewriter.destroy, 'function');

  assert_equals(typeof rewriter.expectedContextLanguages, 'object');
  assert_equals(typeof rewriter.expectedInputLanguages, 'object');
  assert_equals(typeof rewriter.inputQuota, 'number');
  assert_equals(typeof rewriter.outputLanguage, 'object');
  assert_equals(typeof rewriter.sharedContext, 'string');

  assert_equals(typeof rewriter.tone, 'string');
  assert_equals(typeof rewriter.format, 'string');
  assert_equals(typeof rewriter.length, 'string');

  assert_equals(rewriter.tone, 'as-is');
  assert_equals(rewriter.format, 'as-is');
  assert_equals(rewriter.length, 'as-is');
}, 'Rewriter.create() returns a valid object with default options');

promise_test(async () => {
  await testMonitor(createRewriter);
}, 'Rewriter.create() notifies its monitor on downloadprogress');

promise_test(async t => {
  await testCreateMonitorWithAbort(t, Rewriter.create);
}, 'Progress events are not emitted after aborted');

promise_test(async () => {
  const sharedContext = 'This is a shared context string';
  const rewriter = await createRewriter({sharedContext: sharedContext});
  assert_equals(rewriter.sharedContext, sharedContext);
}, 'Rewriter.sharedContext');

promise_test(async () => {
  const rewriter = await createRewriter({ tone: 'more-formal' });
  assert_equals(rewriter.tone, 'more-formal');
}, 'Creating a Rewriter with "more-formal" tone');

promise_test(async () => {
  const rewriter = await createRewriter({ tone: 'more-casual' });
  assert_equals(rewriter.tone, 'more-casual');
}, 'Creating a Rewriter with "more-casual" tone');

promise_test(async () => {
  const rewriter = await createRewriter({ format: 'plain-text' });
  assert_equals(rewriter.format, 'plain-text');
}, 'Creating a Rewriter with "plain-text" format');

promise_test(async () => {
  const rewriter = await createRewriter({ format: 'markdown' });
  assert_equals(rewriter.format, 'markdown');
}, 'Creating a Rewriter with "markdown" format');

promise_test(async () => {
  const rewriter = await createRewriter({ length: 'shorter' });
  assert_equals(rewriter.length, 'shorter');
}, 'Creating a Rewriter with "shorter" length');

promise_test(async () => {
  const rewriter = await createRewriter({ length: 'longer' });
  assert_equals(rewriter.length, 'longer');
}, 'Creating a Rewriter with "longer" length');

promise_test(async () => {
  const rewriter = await createRewriter({expectedInputLanguages: ['en']});
  assert_array_equals(rewriter.expectedInputLanguages, ['en']);
}, 'Rewriter.expectedInputLanguages');

promise_test(async () => {
  const rewriter = await createRewriter({expectedContextLanguages: ['en']});
  assert_array_equals(rewriter.expectedContextLanguages, ['en']);
}, 'Rewriter.expectedContextLanguages');

promise_test(async () => {
  const rewriter = await createRewriter({outputLanguage: 'en'});
  assert_equals(rewriter.outputLanguage, 'en');
}, 'Rewriter.outputLanguage');

promise_test(async (t) => {
  promise_rejects_js(
    t, RangeError,
    createRewriter({ expectedInputLanguages: ['en-abc-invalid'] }));
}, 'Creating Rewriter with malformed language string');

promise_test(async () => {
  const rewriter = await createRewriter({});
  assert_equals(rewriter.expectedInputLanguages, null);
  assert_equals(rewriter.expectedContextLanguages, null);
  assert_equals(rewriter.outputLanguage, null);
}, 'Rewriter optional attributes return null');
