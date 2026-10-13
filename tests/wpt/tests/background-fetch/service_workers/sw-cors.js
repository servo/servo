importScripts('sw-helpers.js');

async function getFetchResult(record) {
  const response = await record.responseReady.catch(() => null);
  if (!response)
    return null;

  return {
    url: response.url,
    status: response.status,
    type: response.type,
    secretHeader: response.headers.get('X-Secret-Header'),
    text: await response.text().catch(() => 'error'),
  };
}

function handleBackgroundFetchEvent(event) {
  event.waitUntil(event.registration.matchAll()
                      .then(records => {
                        if (!records)
                          return [];
                        if (!records.map)
                          return [records];
                        return records;
                      })
                      .then(records => Promise.all(
                                records.map(record => getFetchResult(record))))
                      .then(results => {
                        const registrationCopy =
                            cloneRegistration(event.registration);
                        sendMessageToDocument({
                          type: event.type,
                          eventRegistration: registrationCopy,
                          results
                        });
                      })
                      .catch(error => {
                        sendMessageToDocument({
                          type: event.type,
                          eventRegistration: {},
                          results: [],
                          error: true
                        });
                      }));
}

self.addEventListener('backgroundfetchsuccess', handleBackgroundFetchEvent);
self.addEventListener('backgroundfetchfail', handleBackgroundFetchEvent);
self.addEventListener('backgroundfetchabort', handleBackgroundFetchEvent);
