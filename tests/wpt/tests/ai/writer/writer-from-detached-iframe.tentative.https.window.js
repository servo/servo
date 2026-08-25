// META: title=Writer Detached Iframe
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async (t) => {
  const iframe = document.body.appendChild(document.createElement('iframe'));
  await test_driver.bless('Writer create()', null, iframe.contentWindow);
  iframe.contentWindow.Writer.create();
  iframe.remove();
}, 'Detaching iframe during Writer.create() should not leak memory');

promise_test(async (t) => {
  const iframe = document.body.appendChild(document.createElement('iframe'));
  await test_driver.bless('Writer create()', null, iframe.contentWindow);
  const iframeWindow = iframe.contentWindow;
  const iframeDOMException = iframeWindow.DOMException;
  const iframeWriter = iframeWindow.Writer;
  iframe.remove();

  await promise_rejects_dom(
      t, 'InvalidStateError', iframeDOMException, iframeWriter.create());
}, 'Writer.create() fails on a detached iframe');

promise_test(async (t) => {
  const iframe = document.body.appendChild(document.createElement('iframe'));
  await test_driver.bless('Writer create()', null, iframe.contentWindow);
  const iframeDOMException = iframe.contentWindow.DOMException;
  const writer = await iframe.contentWindow.Writer.create();
  iframe.remove();

  await promise_rejects_dom(
    t, 'InvalidStateError', iframeDOMException, writer.write(kTestPrompt));
}, 'Writer.write() fails on a detached iframe');

promise_test(async (t) => {
  const iframe = document.body.appendChild(document.createElement('iframe'));
  await test_driver.bless('Writer create()', null, iframe.contentWindow);
  const iframeWindow = iframe.contentWindow;
  const iframeDOMException = iframeWindow.DOMException;
  const writer = await iframeWindow.Writer.create();
  iframe.remove();

  assert_throws_dom(
    'InvalidStateError',
    iframeDOMException, () => writer.writeStreaming(kTestPrompt));
}, 'Writer.writeStreaming() fails on a detached iframe');

promise_test(async (t) => {
  const iframe = document.body.appendChild(document.createElement('iframe'));
  await test_driver.bless('Writer create()', null, iframe.contentWindow);
  const writer = await iframe.contentWindow.Writer.create();
  writer.write(kTestPrompt);
  iframe.remove();
}, 'Detaching iframe during Writer.write() should not leak memory');
