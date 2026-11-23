// META: title=Language Model Response JSON Schema - Integer Streaming
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: script=util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  t.add_cleanup(() => session.destroy());
  const streamingResponse = session.promptStreaming(
      'Derive a rating between -10 and 10 from "Absolutely the best meal ever!"',
      {responseConstraint: {type: 'integer'}});
  assert_true(streamingResponse instanceof ReadableStream);
  const response = await readStream(streamingResponse);
  assert_less_than(
      response.length, 64,
      `Response "${response}" should terminate without runaway output`);
  const jsonResponse = parse_json_response(response);
  assert_true(Number.isInteger(jsonResponse), 'Response should be an integer');
}, 'promptStreaming should terminate with an integer json schema constraint.');
