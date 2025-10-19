// META: title=BFCache is allowed for a page with SharedWorker if another client is active.
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/html/browsers/browsing-the-web/back-forward-cache/resources/helper.sub.js
// META: timeout=long

'use strict';

// Check whether the page is BFCached when there is another active client for
// the shared worker.
runBfcacheTest(
    {
      funcBeforeNavigation: async () => {
        globalThis.worker = new SharedWorker('../resources/echo-worker.js');
        await WorkerHelper.pingWorker(globalThis.worker);

        // Create another client in a new window.
        const workerUrl =
            new URL('../resources/echo-worker.js', window.location.href).href;
        const helperPageContent = `
          <script>
            new SharedWorker('${workerUrl}');
            const bc = new BroadcastChannel('worker_ready');
            bc.postMessage('ready');
            bc.close();
          </script>
        `;
        const blob = new Blob([helperPageContent], {type: 'text/html'});
        window.open(URL.createObjectURL(blob), '_blank', 'noopener');
        await new Promise(resolve => {
          const bc = new BroadcastChannel('worker_ready');
          bc.onmessage = e => {
            if (e.data === 'ready') {
              bc.close();
              resolve();
            }
          };
        });
      },
      funcAfterAssertion: async (pageA) => {
        // Confirm that the worker is still there.
        assert_equals(
            await pageA.execute_script(
                () => WorkerHelper.pingWorker(globalThis.worker)),
            'PASS',
            'SharedWorker should still work after restored from BFCache');
      }
    },
    'Eligibility: shared workers with another active client in a separate window');
