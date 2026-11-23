// META: title=Language Model Response Regex - Integer Streaming
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  t.add_cleanup(() => session.destroy());
  const regex = /^-?\d+$/;
  const streamingResponse = session.promptStreaming(
      'Derive a rating between -10 and 10 from "Absolutely the best meal ever!"',
      {responseConstraint: regex});
  assert_true(streamingResponse instanceof ReadableStream);
  const response = await readStream(streamingResponse);
  assert_less_than(
      response.length, 64,
      `Response "${response}" should terminate without runaway output`);
  assert_regexp_match(response, regex);
}, 'promptStreaming should terminate with an unbounded integer regex.');
