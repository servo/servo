// META: title=Language Model Create
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
}, 'Ensure sessions can be created');

promise_test(async t => {
  let session = await createLanguageModel();
  assert_true(session instanceof LanguageModel);

  assert_equals(typeof session.prompt, 'function');
  assert_equals(typeof session.promptStreaming, 'function');
  assert_equals(typeof session.append, 'function');
  assert_equals(typeof session.measureInputUsage, 'function');
  assert_equals(typeof session.clone, 'function');
  assert_equals(typeof session.destroy, 'function');

  assert_equals(typeof session.inputUsage, 'number');
  assert_equals(typeof session.inputQuota, 'number');
  assert_equals(typeof session.topK, 'number');
  assert_equals(typeof session.temperature, 'number');

  assert_equals(typeof session.onquotaoverflow, 'object');
}, 'LanguageModel.create() returns a valid object with default options');

promise_test(async t => {
  await testMonitor(createLanguageModel);
}, 'LanguageModel.create() notifies its monitor on downloadprogress');

promise_test(async t => {
  await testCreateMonitorWithAbort(t, createLanguageModel);
}, 'Progress events are not emitted after aborted.');

promise_test(async t => {
  let session = await createLanguageModel({ topK: 3, temperature: 0.6 });
  assert_true(!!session);
}, 'Create with topK and temperature');

promise_test(async t => {
  let result = createLanguageModel({ topK: 3 });
  await promise_rejects_dom(
      t, 'NotSupportedError', result,
      'Initializing a new session must either specify both topK and temperature, or neither of them.');
}, 'Create with only topK should fail');

promise_test(async t => {
  let result = createLanguageModel({ temperature: 0.5 });
  await promise_rejects_dom(
      t, 'NotSupportedError', result,
      'Initializing a new session must either specify both topK and temperature, or neither of them.');
}, 'Create with only temperature should fail');

promise_test(async t => {
  let result = createLanguageModel({ topK: 3, temperature: -0.5 });
  await promise_rejects_js(t, RangeError, result);
}, 'Create with negative temperature should fail');

promise_test(async t => {
  let result = createLanguageModel({ topK: 0, temperature: 0.5 });
  await promise_rejects_js(t, RangeError, result);
}, 'Create with zero topK should fail');

promise_test(async t => {
  let result = createLanguageModel({ topK: -2, temperature: 0.5 });
  await promise_rejects_js(t, RangeError, result);
}, 'Create with negative topK should fail');

promise_test(async t => {
  let session = await createLanguageModel({ topK: 1.5, temperature: 0.5 });
  assert_true(!!session);
  assert_equals(session.topK, 1);
}, 'Create with fractional topK should be rounded down');

promise_test(async t => {
  let session = await createLanguageModel({
    initialPrompts: [
      {role: 'system', content: 'you are a robot'},
      {role: 'user', content: 'hello'}, {role: 'assistant', content: 'hello'}
    ]
  });
  assert_true(!!session);
}, 'Create with initialPrompts');

promise_test(async t => {
  let session = await createLanguageModel({initialPrompts: []});
  assert_true(!!session);
}, 'Create with empty initialPrompts');

promise_test(async t => {
  let session = await createLanguageModel({
    initialPrompts: [
      {role: 'user', content: 'hello'}, {role: 'assistant', content: 'hello'}
    ]
  });
  assert_true(!!session);
}, 'Create with initialPrompts without system role');

promise_test(async t => {
  let result = createLanguageModel({
    initialPrompts: [
      {role: 'user', content: 'hello'}, {role: 'assistant', content: 'hello'},
      {role: 'system', content: 'you are a robot'}
    ]
  });
  await promise_rejects_js(t, TypeError, result);
}, 'Create with system role not ordered first should fail');

promise_test(async t => {
  let result = createLanguageModel({
    initialPrompts: [
      {role: 'system', content: 'you are a robot'},
      {role: 'system', content: 'you are a kitten'},
      {role: 'user', content: 'hello'}, {role: 'assistant', content: 'hello'}
    ]
  });
  await promise_rejects_js(t, TypeError, result);
}, 'Create multiple system role entries should fail');

promise_test(async (t) => {
  return promise_rejects_js(t, RangeError, LanguageModel.create({
    expectedInputs: [{type: 'text', languages: ['en-abc-invalid']}]
  }));
}, 'LanguageModel.create() rejects when given invalid language tags');

promise_test(async (t) => {
  let session = await LanguageModel.create(
      {expectedInputs: [{type: 'text', languages: ['EN']}]});
  assert_true(!!session);
}, 'LanguageModel.create() canonicalizes language tags');
