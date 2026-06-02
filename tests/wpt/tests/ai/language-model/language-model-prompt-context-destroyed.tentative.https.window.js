// META: title=Language Model Prompt Context Destroyed
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async t => {
  assert_true(!!LanguageModel);
  // Create the iframe and append it to the document.
  const iframe = document.createElement('iframe');
  document.childNodes[document.childNodes.length - 1].appendChild(iframe);

  await test_driver.bless();
  const session = await iframe.contentWindow.LanguageModel.create();
  session.prompt(kTestPrompt);
  // Detach the iframe.
  iframe.remove();
}, 'Detaching iframe while running prompt() should not cause memory leak');
