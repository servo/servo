importScripts('sw-helpers.js');

async function getFetchResult(record) {
  try {
    await record.responseReady;
  } catch (e) {
    return {
      response: false,
      name: e.name,
    };
  }

  return {
    response: true,
  };
}
self.addEventListener('backgroundfetchabort', event => {
  event.waitUntil(
    event.registration.matchAll()
      .then(records =>
            Promise.all(records.map(record => getFetchResult(record))))
      .then(results => sendMessageToDocument({results})));
});
