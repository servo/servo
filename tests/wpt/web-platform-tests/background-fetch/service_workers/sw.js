
importScripts('sw-helpers.js');

async function getFetchResult(record) {
  const response = await record.responseReady.catch(() => null);
  if (!response) return null;

  return {
    url: response.url,
    status: response.status,
    text: await response.text(),
  };
}

function handleBackgroundFetchEvent(event) {
  event.waitUntil(
    event.registration.matchAll()
      .then(records =>
            Promise.all(records.map(record => getFetchResult(record))))
      .then(results => {
        const registrationCopy = cloneRegistration(event.registration);
        sendMessageToDocument(
          { type: event.type, eventRegistration: registrationCopy, results })
      }));
}

self.addEventListener('backgroundfetchsuccess', handleBackgroundFetchEvent);
self.addEventListener('backgroundfetchfail', handleBackgroundFetchEvent);
self.addEventListener('backgroundfetchabort', handleBackgroundFetchEvent);
