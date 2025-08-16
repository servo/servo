// META: title=Proofreader Proofread
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async (t) => {
  const proofreader = await createProofreader();
  let result = await proofreader.proofread('');
  assert_equals(result.correctedInput, '');
  assert_equals(result.corrections, undefined);
}, 'Proofreader.proofread() with an empty input returns an empty text');

promise_test(async (t) => {
  const proofreader = await createProofreader();
  let result = await proofreader.proofread(' ');
  assert_equals(result.correctedInput, ' ');
  assert_equals(result.corrections, undefined);
}, 'Proofreader.proofread() with a whitespace input returns a whitespace text');

promise_test(async (t) => {
  const proofreader = await createProofreader();
  const result = await proofreader.proofread(kTestPrompt);
  assert_not_equals(result.correctedInput, '');
}, 'Proofreader.proofread() with non-empty input returns a non-empty result');

// TODO: add a test for non-empty corrections, kTestPrompt with grammar error.

promise_test(async (t) => {
  await testDestroy(t, createProofreader, {}, [
    proofreader => proofreader.proofread(kTestPrompt)
  ]);
}, 'Calling Proofreader.destroy() aborts calls to proofread');

promise_test(async t => {
  await testCreateAbort(t, createProofreader, {}, [
    proofreader => proofreader.proofread(kTestPrompt)
  ]);
}, 'Proofreader.create()\'s abort signal destroys its Proofreader after creation.');

promise_test(async () => {
  const proofreader = await createProofreader();
  const result = await proofreader.proofread(kTestPrompt);
  assert_equals(typeof result, 'object');
}, 'Simple Proofreader.proofread() call');

promise_test(async () => {
  const proofreader = await createProofreader();
  await Promise.all(
    [proofreader.proofread(kTestPrompt), proofreader.proofread(kTestPrompt)]);
}, 'Multiple Proofreader.proofread() calls are resolved successfully');
