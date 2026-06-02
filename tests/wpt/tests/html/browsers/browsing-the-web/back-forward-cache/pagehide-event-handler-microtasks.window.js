// META: title=Assure microtasks posted by pagehide event handler are dispatched as page is BFCached
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/utils.js
// META: script=./resources/test-helper.js

'use strict';

promise_test(async t => {
  const uuid = token();
  const url = remoteExecutorUrl(uuid, {protocol: 'http:'});
  const win = window.open(url, '_blank', 'noopener');
  const context = new RemoteContext(uuid);

  // Navigate within the same eval, as (1) we can't navigate externally
  // because of potential race against the eval, and (2) we can't navigate
  // as a separate eval step as it would block on this one returning,
  // which would only happen upon `pagehide`.
  const {persisted} = await context.execute_script(
      () => new Promise(resolve => {
        window.addEventListener('pagehide', async (event) => {
          await Promise.resolve();  // ... so the rest will run as a microtask.
          resolve({persisted: event.persisted});
        }, false);
        location.href += '&navigated';
      }));

  assert_true(persisted);  // Validate we're BFCached.
}, 'Assure microtasks posted by pagehide event handler are dispatched');
