// META: title=Language Model Prompt Empty Input
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async (t) => {
  await ensureLanguageModel();
  const model = await createLanguageModel();

  // null and undefined are coerced to the strings "null" and "undefined" by the
  // IDL bindings.
  assert_regexp_match(await model.prompt(null), /null/);
  assert_regexp_match(await model.prompt(undefined), /undefined/);

  // Empty string is allowed even when context is empty.
  assert_equals(typeof await model.prompt(""), "string");

  // Empty sequence [] is allowed even when context is empty.
  assert_equals(typeof await model.prompt([]), "string");

  // Nested empty content sequence is allowed.
  assert_equals(typeof await model.prompt([{ role: 'user', content: [] }]), "string");

  // Nested structured message with empty text is allowed.
  assert_equals(typeof await model.prompt([{ role: 'user', content: [{ type: 'text', value: '' }] }]), "string");
}, "LanguageModel.prompt() allows empty or coerced inputs");

promise_test(async (t) => {
  await ensureLanguageModel();
  const model = await createLanguageModel();

  assert_equals(await model.append(null), undefined);
  assert_equals(await model.append(undefined), undefined);
  assert_equals(await model.append(""), undefined);
  assert_equals(await model.append([]), undefined);
  assert_equals(await model.append([{ role: 'user', content: [] }]), undefined);
  assert_equals(await model.append([{ role: 'user', content: [{ type: 'text', value: '' }] }]), undefined);
}, "LanguageModel.append() allows empty or coerced inputs");

promise_test(async (t) => {
  await ensureLanguageModel();
  const model = await createLanguageModel();

  assert_true(model.promptStreaming(null) instanceof ReadableStream);
  assert_true(model.promptStreaming(undefined) instanceof ReadableStream);
  assert_true(model.promptStreaming("") instanceof ReadableStream);
  assert_true(model.promptStreaming([]) instanceof ReadableStream);
  assert_true(model.promptStreaming([{ role: 'user', content: [] }]) instanceof ReadableStream);
  assert_true(model.promptStreaming([{ role: 'user', content: [{ type: 'text', value: '' }] }]) instanceof ReadableStream);
}, "LanguageModel.promptStreaming() allows empty or coerced inputs");
