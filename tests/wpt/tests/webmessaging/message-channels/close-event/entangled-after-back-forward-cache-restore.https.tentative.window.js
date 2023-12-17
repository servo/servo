// META: timeout=long
// META: title=Confirm close event is not fired when the page enters BFCache and MessagePort still works after the page is restored.
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=/service-workers/service-worker/resources/test-helpers.sub.js
// META: script=/html/browsers/browsing-the-web/back-forward-cache/resources/rc-helper.js

promise_test(async t => {
  // Register a service worker.
  const scope =
      '/html/browsers/browsing-the-web/remote-context-helper/resources'
  const workerUrl =
      `/html/browsers/browsing-the-web/back-forward-cache/resources/` +
      `service-worker.js?pipe=header(Service-Worker-Allowed,${scope})`;
  const registration =
      await service_worker_unregister_and_register(t, workerUrl, scope);
  t.add_cleanup(_ => registration.unregister());
  await wait_for_state(t, registration.installing, 'activated');

  // Open a window with noopener so that BFCache will work.
  const rcHelper = new RemoteContextHelper();
  const rc1 = await rcHelper.addWindow(
      /*extraConfig=*/ null, /*options=*/ {features: 'noopener'});

  // Confirm the page is controlled.
  assert_true(
      await rc1.executeScript(
          () => (navigator.serviceWorker.controller !== null)),
      'The page should be controlled before navigation');

  // Send MessagePort to the service worker.
  await rc1.executeScript(() => {
    const {port1, port2} = new MessageChannel();
    port1.start();
    const ctrl = navigator.serviceWorker.controller;
    ctrl.postMessage({type: 'storeMessagePort'}, [port2]);
    self.waitForMessage = (sentMessage) => {
      return new Promise(resolve => {
        port1.addEventListener('message', (event) => {
          resolve(event.data);
        });
        port1.postMessage(sentMessage);
      });
    };
  });

  // Verify that the page was BFCached.
  await assertBFCacheEligibility(rc1, /*shouldRestoreFromBFCache=*/ true);

  // Confirm MessagePort can still work after the page is restored from
  // BFCache.
  assert_equals(
      await rc1.executeScript(
          async () =>
              await self.waitForMessage('Confirm the ports can communicate')),
      'Receive message');

  // Confirm the close event was not fired.
  assert_false(await rc1.executeScript(
      async () =>
          await self.waitForMessage('Ask if the close event was fired')));
}, 'MessagePort still works after the page is restored from BFCache');