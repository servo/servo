// META: title=Close event test when an entangled port is explicitly closed.
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=resources/helper.js

async_test(t => {
  const channel = new MessageChannel();
  channel.port1.start();
  channel.port2.start();
  channel.port2.onclose = t.step_func_done();
  channel.port1.close();
}, 'Close event on port2 is fired when port1 is explicitly closed');

async_test(t => {
  const channel = new MessageChannel();
  channel.port1.start();
  channel.port2.start();
  channel.port1.onclose =
      t.unreached_func('Should not fire a close event on port1');
  channel.port1.close();
  t.step_timeout(t.step_func_done(), 1000);
}, 'Close event on port1 is not fired when port1 is explicitly closed');

promise_test(async t => {
  const rc = await addWindow();
  const waitForPort = expectMessagePortFromWindow(window);
  await createMessageChannelAndSendPort(rc);
  const closeEventPromise = createCloseEventPromise(await waitForPort);
  rc.executeScript(() => {
    window.closePort();
  });
  await closeEventPromise;
}, 'Close event on port2 is fired when port1, which is in a different window, is explicitly closed.')
