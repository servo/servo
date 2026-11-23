// META: title=Language Model - Thinking Effort
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

const kValidThinkingEfforts = ['none', 'low', 'medium', 'high'];
const kInvalidThinkingEfforts = ['invalid', '', 'Low', null, 123];

promise_test(async t => {
  for (const effort of kValidThinkingEfforts) {
    const result = await LanguageModel.availability({thinkingEffort: effort});
    assert_in_array(
        result, kValidAvailabilities,
        `availability() returned valid result for thinkingEffort=${effort}`);
  }
}, 'LanguageModel.availability() accepts all valid thinkingEffort values');

promise_test(async t => {
  for (const effort of kInvalidThinkingEfforts) {
    await promise_rejects_js(
        t, TypeError, LanguageModel.availability({thinkingEffort: effort}),
        `availability() should reject thinkingEffort=${
            JSON.stringify(effort)}`);
  }
}, 'LanguageModel.availability() rejects invalid thinkingEffort values with TypeError');

promise_test(async t => {
  await ensureLanguageModel();

  const defaultSession = await createLanguageModel();
  assert_equals(defaultSession.thinkingEffort, 'none');
  assert_throws_js(TypeError, () => {
    defaultSession.thinkingEffort = 'high';
  });
  const clonedDefault = await defaultSession.clone();
  assert_equals(clonedDefault.thinkingEffort, 'none');
  clonedDefault.destroy();
  defaultSession.destroy();

  for (const effort of kValidThinkingEfforts) {
    const availability =
        await LanguageModel.availability({thinkingEffort: effort});
    if (effort === 'none') {
      assert_not_equals(availability, 'unavailable');
    }
    if (availability === 'unavailable') {
      await promise_rejects_dom(t, 'NotSupportedError',
                                createLanguageModel({thinkingEffort: effort}));
      continue;
    }
    const session = await createLanguageModel({thinkingEffort: effort});
    assert_equals(session.thinkingEffort, effort);
    const cloned = await session.clone();
    assert_equals(cloned.thinkingEffort, effort);
    cloned.destroy();
    session.destroy();
  }
}, 'LanguageModel.create() defaults readonly thinkingEffort to none and preserves valid values on clone()');

promise_test(async t => {
  for (const effort of kInvalidThinkingEfforts) {
    await promise_rejects_js(
        t, TypeError, createLanguageModel({thinkingEffort: effort}),
        `create() should reject thinkingEffort=${JSON.stringify(effort)}`);
  }
}, 'LanguageModel.create() rejects invalid thinkingEffort values with TypeError');

promise_test(async t => {
  await ensureLanguageModel();

  const lowAvailability =
      await LanguageModel.availability({thinkingEffort: 'low'});
  const mediumAvailability =
      await LanguageModel.availability({thinkingEffort: 'medium'});
  const highAvailability =
      await LanguageModel.availability({thinkingEffort: 'high'});
  if (lowAvailability === 'unavailable' ||
      mediumAvailability === 'unavailable' ||
      highAvailability === 'unavailable') {
    const session = await createLanguageModel();
    t.add_cleanup(() => session.destroy());
    assert_equals(session.thinkingEffort, 'none');

    const promptReply =
        await session.prompt('Reply with OK.', {thinkingEffort: 'none'});
    assert_equals(typeof promptReply, 'string');
    assert_greater_than(promptReply.length, 0);

    const streamedReply = await readStream(
        session.promptStreaming('Reply with OK.', {thinkingEffort: 'none'}));
    assert_equals(typeof streamedReply, 'string');
    assert_greater_than(streamedReply.length, 0);

    const baselineUsage = await session.measureContextUsage(kTestPrompt);
    assert_greater_than(baselineUsage, 0);
    const noneUsage = await session.measureContextUsage(
        kTestPrompt, {thinkingEffort: 'none'});
    assert_equals(noneUsage, baselineUsage);
    assert_equals(session.thinkingEffort, 'none');
    return;
  }

  const session = await createLanguageModel({thinkingEffort: 'medium'});
  t.add_cleanup(() => session.destroy());
  assert_equals(session.thinkingEffort, 'medium');

  const promptReply =
      await session.prompt('Reply with OK.', {thinkingEffort: 'none'});
  assert_equals(typeof promptReply, 'string');
  assert_greater_than(promptReply.length, 0);
  assert_equals(session.thinkingEffort, 'medium');

  const streamedReply = await readStream(
      session.promptStreaming('Reply with OK.', {thinkingEffort: 'low'}));
  assert_equals(typeof streamedReply, 'string');
  assert_greater_than(streamedReply.length, 0);
  assert_equals(session.thinkingEffort, 'medium');

  const baselineUsage = await session.measureContextUsage(kTestPrompt);
  assert_greater_than(baselineUsage, 0);
  for (const effort of kValidThinkingEfforts) {
    const usage = await session.measureContextUsage(kTestPrompt,
                                                    {thinkingEffort: effort});
    assert_equals(usage, baselineUsage);
    assert_equals(session.thinkingEffort, 'medium');
  }
}, 'Per-call thinkingEffort overrides work on prompt(), promptStreaming(), and measureContextUsage() without mutating session.thinkingEffort');

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  t.add_cleanup(() => session.destroy());

  for (const effort of kInvalidThinkingEfforts) {
    await promise_rejects_js(
        t, TypeError, session.prompt(kTestPrompt, {thinkingEffort: effort}),
        `prompt() should reject thinkingEffort=${JSON.stringify(effort)}`);
    assert_throws_js(
        TypeError,
        () => session.promptStreaming(kTestPrompt, {thinkingEffort: effort}),
        `promptStreaming() should throw for thinkingEffort=${
            JSON.stringify(effort)}`);
    await promise_rejects_js(
        t, TypeError,
        session.measureContextUsage(kTestPrompt, {thinkingEffort: effort}),
        `measureContextUsage() should reject thinkingEffort=${
            JSON.stringify(effort)}`);
  }
}, 'Per-call invalid thinkingEffort values reject or throw TypeError');
