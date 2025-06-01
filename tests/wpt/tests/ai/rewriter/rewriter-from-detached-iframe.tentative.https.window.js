// META: title=Rewriter Detached Iframe
// META: script=/resources/testdriver.js
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async (t) => {
  const iframe = document.body.appendChild(document.createElement('iframe'));
  await test_driver.bless('Rewriter create()', null, iframe.contentWindow);
  iframe.contentWindow.Rewriter.create();
  iframe.remove();
}, 'Detaching iframe during Rewriter.create() should not leak memory');

promise_test(async (t) => {
  const iframe = document.body.appendChild(document.createElement('iframe'));
  await test_driver.bless('Rewriter create()', null, iframe.contentWindow);
  const iframeWindow = iframe.contentWindow;
  const iframeDOMException = iframeWindow.DOMException;
  const iframeRewriter = iframeWindow.Rewriter;
  iframe.remove();

  await promise_rejects_dom(
      t, 'InvalidStateError', iframeDOMException, iframeRewriter.create());
}, 'Rewriter.create() fails on a detached iframe.');

promise_test(async (t) => {
  const iframe = document.body.appendChild(document.createElement('iframe'));
  await test_driver.bless('Rewriter create()', null, iframe.contentWindow);
  const iframeDOMException = iframe.contentWindow.DOMException;
  const rewriter = await iframe.contentWindow.Rewriter.create();
  iframe.remove();

  await promise_rejects_dom(
      t, 'InvalidStateError', iframeDOMException, rewriter.rewrite('hello'));
}, 'Rewriter.rewrite() fails on a detached iframe.');

promise_test(async (t) => {
  const iframe = document.body.appendChild(document.createElement('iframe'));
  await test_driver.bless('Rewriter create()', null, iframe.contentWindow);
  const iframeWindow = iframe.contentWindow;
  const iframeDOMException = iframeWindow.DOMException;
  const rewriter = await iframeWindow.Rewriter.create();
  iframe.remove();

  assert_throws_dom(
    'InvalidStateError',
    iframeDOMException, () => rewriter.rewriteStreaming('hello'));
}, 'Rewriter.rewriteStreaming() fails on a detached iframe.');

promise_test(async (t) => {
  const iframe = document.body.appendChild(document.createElement('iframe'));
  await test_driver.bless('Rewriter create()', null, iframe.contentWindow);
  const rewriter = await iframe.contentWindow.Rewriter.create();
  rewriter.rewrite('hello');
  iframe.remove();
}, 'Detaching iframe during Rewriter.rewrite() should not leak memory');
