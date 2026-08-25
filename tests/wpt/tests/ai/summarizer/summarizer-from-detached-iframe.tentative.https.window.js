// META: title=Summarizer Detached Iframe
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async (t) => {
  const iframe = document.body.appendChild(document.createElement('iframe'));
  await test_driver.bless('Summarizer create()', null, iframe.contentWindow);
  iframe.contentWindow.Summarizer.create();
  iframe.remove();
}, 'Detaching iframe during Summarizer.create() should not leak memory');

promise_test(async (t) => {
  const iframe = document.body.appendChild(document.createElement('iframe'));
  await test_driver.bless('Summarizer create()', null, iframe.contentWindow);
  const iframeWindow = iframe.contentWindow;
  const iframeDOMException = iframeWindow.DOMException;
  const iframeSummarizer = iframeWindow.Summarizer;
  iframe.remove();

  await promise_rejects_dom(
      t, 'InvalidStateError', iframeDOMException, iframeSummarizer.create());
}, 'Summarizer.create() fails on a detached iframe');

promise_test(async (t) => {
  const iframe = document.body.appendChild(document.createElement('iframe'));
  await test_driver.bless('Summarizer create()', null, iframe.contentWindow);
  const iframeDOMException = iframe.contentWindow.DOMException;
  const summarizer = await iframe.contentWindow.Summarizer.create();
  iframe.remove();

  await promise_rejects_dom(
    t, 'InvalidStateError', iframeDOMException, summarizer.summarize(kTestPrompt));
}, 'Summarizer.summarize() fails on a detached iframe');

promise_test(async (t) => {
  const iframe = document.body.appendChild(document.createElement('iframe'));
  await test_driver.bless('Summarizer create()', null, iframe.contentWindow);
  const iframeWindow = iframe.contentWindow;
  const iframeDOMException = iframeWindow.DOMException;
  const summarizer = await iframeWindow.Summarizer.create();
  iframe.remove();

  assert_throws_dom(
    'InvalidStateError',
    iframeDOMException, () => summarizer.summarizeStreaming(kTestPrompt));
}, 'Summarizer.summarizeStreaming() fails on a detached iframe');

promise_test(async (t) => {
  const iframe = document.body.appendChild(document.createElement('iframe'));
  await test_driver.bless('Summarizer create()', null, iframe.contentWindow);
  const summarizer = await iframe.contentWindow.Summarizer.create();
  summarizer.summarize(kTestPrompt);
  iframe.remove();
}, 'Detaching iframe during Summarizer.summarize() should not leak memory');
