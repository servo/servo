// Functions available by default in the executor.

'use strict';

let executorStartEvent = null;

function requestExecutor() {
  const params = new URLSearchParams(location.search);
  const startOn = params.get('startOn');

  if (startOn) {
    addEventListener(startOn, (e) => {
      executorStartEvent = e;
      startExecutor();
    });
  } else {
    startExecutor();
  }
}

function addScript(url) {
  const script = document.createElement('script');
  script.src = url;
  const promise = new Promise((resolve, reject) => {
    script.onload = () => resolve(url);
    script.onerror = (e) => reject(e);
  });
  document.body.appendChild(script);
  return promise;
}

/**
 * Suspends the executor and executes the function in `fnString` when it has
 * suspended. Installs a pageshow handler to resume the executor if the
 * document is BFCached. Installs a hashchange handler to detect when the
 * navigation did not change documents.
 *
 * This returns nothing because fn is invoke after waiting for the document to
 * be suspended. If we were to return a promise, the executor could not suspend
 * until that promise resolved but the promise cannot resolve until the executor
 * is suspended. This could be avoided by adding support
 * directly in the dispatcher for tasks suspend immediately after execution.
 *
 * @param {string} fnString A stringified function to be executed.
 * @param {any[]} args The arguments to pass to the function.
 */
function executeScriptToNavigate(fnString, args) {
  // Only one of these listeners should run.
  const controller = new AbortController();
  window.addEventListener('pageshow', (event) => {
    controller.abort();
    executor.resume();
  }, {signal: controller.signal, once: true});
  window.addEventListener('hashchange', (event) => {
    controller.abort();
    const oldURLObject = new URL(event.oldURL);
    const newURLObject = new URL(event.newURL);
    oldURLObject.hash = '';
    newURLObject.hash = '';
    // If only the hash-fragment changed then the navigation was
    // same-document and we should resume the executor.
    if (oldURLObject.toString() == newURLObject.toString()) {
      executor.resume();
    }
  }, {signal: controller.signal, once: true});

  executor.suspend(() => {
    eval(fnString).apply(null, args);
  });
}
