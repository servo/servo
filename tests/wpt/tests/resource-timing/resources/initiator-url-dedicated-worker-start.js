// Starts the dedicated worker under test as both a classic and a module worker.
// Included by the test document via <script src>. Requires
// /common/get-host-info.sub.js and test-initiator.js (getUrl) to be loaded
// first.
const workerScriptUrl = label =>
    getUrl('/resource-timing/resources/initiator-url-worker.js?label=' + label);

// Start both a classic and a module worker.
new Worker(workerScriptUrl('classic-worker-from-js'));
new Worker(workerScriptUrl('module-worker-from-js'), {type: 'module'});

function startDedicatedWorkers() {
  new Worker(workerScriptUrl('classic-worker-from-setTimeout'));
  new Worker(workerScriptUrl('module-worker-from-setTimeout'),
             {type: 'module'});
}
