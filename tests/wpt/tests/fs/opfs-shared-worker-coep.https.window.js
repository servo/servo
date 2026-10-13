// META: title=OPFS availability in SharedWorker served with COOP/COEP headers
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js

'use strict';

promise_test(async t => {
  const rcHelper = new RemoteContextHelper({origin: 'HTTPS_REMOTE_ORIGIN'});
  const main = await rcHelper.addWindow({
    headers: [
      ['cross-origin-opener-policy', 'same-origin'],
      ['cross-origin-embedder-policy', 'require-corp'],
    ],
  });

  const worker = await main.addSharedWorker('workerVar');

  const isDirectoryHandle = await worker.executeScript(async () => {
    const dir = await navigator.storage.getDirectory();
    return dir instanceof FileSystemDirectoryHandle;
  });

  assert_true(isDirectoryHandle, 'navigator.storage.getDirectory() should return a FileSystemDirectoryHandle in a COOP/COEP SharedWorker');
}, 'SharedWorker served with COOP/COEP headers can use OPFS');
