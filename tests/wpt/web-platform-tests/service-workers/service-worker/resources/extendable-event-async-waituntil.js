// controlled by 'init'/'done' messages.
var resolveLockPromise;
var port;

self.addEventListener('message', function(event) {
    var waitPromise;
    var resolveTestPromise;

    switch (event.data.step) {
      case 'init':
        event.waitUntil(new Promise((res) => { resolveLockPromise = res; }));
        port = event.data.port;
        break;
      case 'done':
        resolveLockPromise();
        break;
      case 'no-current-extension-different-task':
        async_task_waituntil(event).then(reportResultExpecting('InvalidStateError'));
        break;
      case 'no-current-extension-different-microtask':
        async_microtask_waituntil(event).then(reportResultExpecting('InvalidStateError'));
        break;
      case 'current-extension-different-task':
        event.waitUntil(new Promise((res) => { resolveTestPromise = res; }));
        async_task_waituntil(event).then(reportResultExpecting('OK')).then(resolveTestPromise);
        break;
      case 'current-extension-expired-same-microtask-turn':
        waitPromise = Promise.resolve();
        event.waitUntil(waitPromise);
        waitPromise.then(() => { return sync_waituntil(event); })
          .then(reportResultExpecting('OK'))
        break;
      case 'current-extension-expired-same-microtask-turn-extra':
        // The promise handler queues a new microtask *after* the check for new
        // extensions was performed.
        waitPromise = Promise.resolve();
        event.waitUntil(waitPromise);
        waitPromise.then(() => { return async_microtask_waituntil(event); })
          .then(reportResultExpecting('InvalidStateError'))
        break;
      case 'current-extension-expired-different-task':
        event.waitUntil(Promise.resolve());
        async_task_waituntil(event).then(reportResultExpecting('InvalidStateError'));
        break;
      case 'script-extendable-event':
        new_event_waituntil().then(reportResultExpecting('InvalidStateError'));
        break;
    }
    event.source.postMessage('ACK');
  });

self.addEventListener('fetch', function(event) {
    if (event.request.url.indexOf('pending-respondwith-async-waituntil') != -1) {
      var resolveFetch;
      let response = new Promise((res) => { resolveFetch = res; });
      event.respondWith(response);
      async_task_waituntil(event)
        .then(reportResultExpecting('OK'))
        .then(() => { resolveFetch(new Response('OK')); });
    } else if (event.request.url.indexOf('respondwith-microtask-sync-waituntil') != -1) {
      response = Promise.resolve(new Response('RESP'));
      event.respondWith(response);
      response.then(() => { return sync_waituntil(event); })
        .then(reportResultExpecting('OK'))
    } else if (event.request.url.indexOf('respondwith-microtask-async-waituntil') != -1) {
      response = Promise.resolve(new Response('RESP'));
      event.respondWith(response);
      response.then(() => { return async_microtask_waituntil(event); })
        .then(reportResultExpecting('InvalidStateError'))
    }
  });

function reportResultExpecting(expectedResult) {
  return function (result) {
    port.postMessage({result : result, expected: expectedResult});
    return result;
  };
}

function sync_waituntil(event) {
  return new Promise((res, rej) => {
    try {
        event.waitUntil(Promise.resolve());
        res('OK');
      } catch (error) {
        res(error.name);
      }
  });
}

function new_event_waituntil() {
  return new Promise((res, rej) => {
    try {
      let e = new ExtendableEvent('foo');
      e.waitUntil(new Promise(() => {}));
      res('OK');
    } catch (error) {
      res(error.name);
    }
  });
}

function async_microtask_waituntil(event) {
  return new Promise((res, rej) => {
    Promise.resolve().then(() => {
      try {
        event.waitUntil(Promise.resolve());
        res('OK');
      } catch (error) {
        res(error.name);
      }
    });
  });
}

function async_task_waituntil(event) {
  return new Promise((res, rej) => {
    setTimeout(() => {
      try {
        event.waitUntil(Promise.resolve());
        res('OK');
      } catch (error) {
        res(error.name);
      }
    }, 0);
  });
}
