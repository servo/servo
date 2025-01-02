// META: title=Close event test when the document is destroyed.
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=resources/helper.js

promise_test(async t => {
  const rc = await addWindow();
  const waitForPort = expectMessagePortFromWindow(window);
  await createMessageChannelAndSendPort(rc);
  const closeEventPromise = createCloseEventPromise(await waitForPort);
  rc.navigateToNew();
  await closeEventPromise;
}, 'The context is navigated to a new document and a close event is fired.')

promise_test(async t => {
  const rc = await addWindow();
  const waitForPort = expectMessagePortFromWindow(window);
  await createMessageChannelAndSendPort(rc);
  const closeEventPromise = createCloseEventPromise(await waitForPort);
  rc.executeScript(() => window.close());
  await closeEventPromise;
}, 'The window is closed and a close event is fired.')

promise_test(async t => {
  let iframe;
  const waitForLoad = new Promise(resolve => {
    iframe = document.createElement('iframe');
    iframe.onload = resolve;
    document.documentElement.appendChild(iframe);
  });
  await waitForLoad;

  const waitForPort = expectMessagePortFromWindow(iframe.contentWindow);
  const {port1, port2} = new MessageChannel();
  port1.start();
  iframe.contentWindow.postMessage('', '*', [port2]);
  await waitForPort;
  const closeEventPromise = createCloseEventPromise(port1);
  iframe.remove();
  await closeEventPromise;
}, 'The iframe is deleted and a close event is fired.')
