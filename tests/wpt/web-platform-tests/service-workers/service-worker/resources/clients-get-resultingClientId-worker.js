let savedPort = null;
let savedResultingClientId = null;

async function destroyResultingClient(e) {
  const outer = await self.clients.matchAll({ type: 'window', includeUncontrolled: true })
    .then((clientList) => {
    for (let c of clientList) {
      if (c.url.endsWith('clients-get.https.html')) {
        c.focus();
        return c;
      }
    }
  });

  const p = new Promise(resolve => {
    function resultingClientDestroyed(evt) {
      if (evt.data.msg == 'resultingClientDestroyed') {
        self.removeEventListener('message', resultingClientDestroyed);
        resolve(outer);
      }
    }

    self.addEventListener('message', resultingClientDestroyed);
  });

  outer.postMessage({ msg: 'destroyResultingClient' });

  return await p;
}

self.addEventListener('fetch', async (e) => {
  let { resultingClientId } = e;
  savedResultingClientId = resultingClientId;

  if (e.request.url.endsWith('simple.html?fail')) {
    e.waitUntil(new Promise(async (resolve) => {
        let outer = await destroyResultingClient(e);

        outer.postMessage({ msg: 'resultingClientDestroyedAck',
                            resultingDestroyedClientId: savedResultingClientId });
        resolve();
    }));
  } else {
    e.respondWith(fetch(e.request));
  }
});

self.addEventListener('message', (e) => {
  let { msg, port, resultingClientId } = e.data;
  savedPort = savedPort || port;

  if (msg == 'getIsResultingClientUndefined') {
    self.clients.get(resultingClientId).then((client) => {
      let isUndefined = typeof client == 'undefined';
      savedPort.postMessage({ msg: 'getIsResultingClientUndefined',
        isResultingClientUndefined: isUndefined });
    });
  }

  if (msg == 'getResultingClientId') {
    savedPort.postMessage({ msg: 'getResultingClientId',
      resultingClientId: savedResultingClientId });
  }
});
