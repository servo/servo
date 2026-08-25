// META: title=Language Model Response Regex - Max Length
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../../../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  await ensureLanguageModel();
  const session = await createLanguageModel();
  const regex = /^.{0,125}$/;
  const response = await session.prompt(
      'Write a short story that is no more than 125 characters.',
      {responseConstraint: regex});
  assert_true(typeof response === 'string');
  assert_true(regex.test(response),
              `Response "${response}" should match regex ${regex}`);
  assert_less_than_equal(response.length, 125,
                         'Response length should be <= 125');
}, 'Prompt should work with a max length regex constraint.');
