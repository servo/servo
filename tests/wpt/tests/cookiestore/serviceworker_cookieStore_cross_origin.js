self.GLOBAL = {
  isWindow: () => false,
  isWorker: () => false,
  isShadowRealm: () => false,
};

self.addEventListener('message', async event => {
  if (event.data.op === 'get-cookies') {
    const workerCookies = await cookieStore.getAll();
    event.ports[0].postMessage({ workerCookies }, {
        domain: event.origin,
    });
  }
});
