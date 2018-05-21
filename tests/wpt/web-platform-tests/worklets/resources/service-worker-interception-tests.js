function openWindow(url) {
  return new Promise(resolve => {
      let win = window.open(url, '_blank');
      add_result_callback(() => win.close());
      window.onmessage = e => {
        assert_equals(e.data, 'LOADED');
        resolve(win);
      };
    });
}

// Runs a series of tests related to service worker interception for a worklet.
//
// Usage:
// runServiceWorkerInterceptionTests("paint");
function runServiceWorkerInterceptionTests(worklet_type) {
    const worklet = get_worklet(worklet_type);

    // Tests that a worklet should be served by the owner document's service
    // worker.
    //
    // [Current document] registers a service worker for Window's URL.
    // --(open)--> [Window] should be controlled by the service worker.
    //   --(addModule)--> [Worklet] should be served by the service worker.
    promise_test(t => {
        const kWindowURL = 'resources/addmodule-window.html';
        const kServiceWorkerScriptURL = 'resources/service-worker.js';
        // This doesn't contain the 'resources/' prefix because this will be
        // imported from a html file under resources/.
        const kWorkletScriptURL = 'non-existent-worklet-script.js';

        return service_worker_unregister_and_register(
            t, kServiceWorkerScriptURL, kWindowURL)
          .then(r => {
              add_result_callback(() => r.unregister());
              return wait_for_state(t, r.installing, 'activated');
            })
          .then(() => openWindow(kWindowURL))
          .then(win => {
              assert_not_equals(win.navigator.serviceWorker.controller, null,
                                'The document should be controlled.');
              const promise = new Promise(r => window.onmessage = r);
              // The worklet script on kWorkletScriptURL doesn't exist but the
              // service worker serves it, so the addModule() should succeed.
              win.postMessage({ type: worklet_type,
                                script_url: kWorkletScriptURL }, '*');
              return promise;
            })
          .then(msg_event => assert_equals(msg_event.data, 'RESOLVED'));
    }, 'addModule() on a controlled document should be intercepted by a ' +
       'service worker.');

    // Tests that a worklet should not be served by a service worker other than
    // the owner document's service worker.
    //
    // [Current document] registers a service worker for Worklet's URL.
    // --(open)--> [Window] should not be controlled by the service worker.
    //   --(addModule)--> [Worklet] should not be served by the service worker.
    promise_test(t => {
        const kWindowURL = 'resources/addmodule-window.html';
        const kServiceWorkerScriptURL = 'resources/service-worker.js';
        // This doesn't contain the 'resources/' prefix because this will be
        // imported from a html file under resources/.
        const kWorkletScriptURL = 'non-existent-worklet-script.js';

        return service_worker_unregister_and_register(
            t, kServiceWorkerScriptURL, 'resources/' + kWorkletScriptURL)
          .then(r =>  {
              add_result_callback(() => r.unregister());
              return wait_for_state(t, r.installing, 'activated');
            })
          .then(() => openWindow(kWindowURL))
          .then(win => {
              assert_equals(win.navigator.serviceWorker.controller, null,
                            'The document should not be controlled.');
              const promise = new Promise(r => window.onmessage = r);
              // The worklet script on kWorkletScriptURL doesn't exist and the
              // service worker doesn't serve it, so the addModule() should
              // fail.
              win.postMessage({ type: worklet_type,
                                script_url: kWorkletScriptURL }, '*');
              return promise;
            })
          .then(msg_event => assert_equals(msg_event.data, 'REJECTED'));
    }, 'addModule() on a non-controlled document should not be intercepted ' +
       'by a service worker even if the script is under the service worker ' +
       'scope.');

    // Tests that static import should be served by the owner document's service
    // worker.
    //
    // [Current document] registers a service worker for Window's URL.
    // --(open)--> [Window] should be controlled by the service worker.
    //   --(addModule)--> [Worklet] should be served by the service worker.
    //     --(static import)--> [Script] should be served by the service worker.
    promise_test(t => {
        const kWindowURL = 'resources/addmodule-window.html';
        const kServiceWorkerScriptURL = 'resources/service-worker.js';
        // This doesn't contain the 'resources/' prefix because this will be
        // imported from a html file under resources/.
        const kWorkletScriptURL = 'import-non-existent-worklet-script.js';

        return service_worker_unregister_and_register(
            t, kServiceWorkerScriptURL, kWindowURL)
          .then(r => {
              add_result_callback(() => r.unregister());
              return wait_for_state(t, r.installing, 'activated');
            })
          .then(() => openWindow(kWindowURL))
          .then(win => {
              assert_not_equals(win.navigator.serviceWorker.controller, null,
                                'The document should be controlled.');
              const promise = new Promise(r => window.onmessage = r);
              // A script statically imported by the worklet doesn't exist but
              // the service worker serves it, so the addModule() should
              // succeed.
              win.postMessage({ type: worklet_type,
                                script_url: kWorkletScriptURL }, '*');
              return promise;
            })
          .then(msg_event => assert_equals(msg_event.data, 'RESOLVED'));
    }, 'Static import should be intercepted by a service worker.');
}
