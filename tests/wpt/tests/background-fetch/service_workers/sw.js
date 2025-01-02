
importScripts('sw-helpers.js');

async function getFetchResult(record) {
  const response = await record.responseReady.catch(() => null);
  if (!response) return null;

  return {
    url: response.url,
    status: response.status,
    text: await response.text().catch(() => 'error'),
  };
}

function handleBackgroundFetchEvent(event) {
  let matchFunction = null;

  switch (event.registration.id) {
    case 'matchexistingrequest':
      matchFunction = event.registration.match.bind(
          event.registration, '/background-fetch/resources/feature-name.txt');
      break;
    case 'matchexistingrequesttwice':
      matchFunction = (async () => {
        const match1 = await event.registration.match('/background-fetch/resources/feature-name.txt');
        const match2 = await event.registration.match('/background-fetch/resources/feature-name.txt');
        return [match1, match2];
    }).bind(event.registration);
      break;
    case 'matchmissingrequest':
      matchFunction = event.registration.match.bind(
          event.registration, '/background-fetch/resources/missing.txt');
      break;
    default:
      matchFunction = event.registration.matchAll.bind(event.registration);
      break;
  }

  event.waitUntil(
    matchFunction()
      // Format `match(All)?` function results.
      .then(records => {
        if (!records) return [];  // Nothing was matched.
        if (!records.map) return [records];  // One entry was returned.
        return records;  // Already in a list.
      })
      // Extract responses.
      .then(records =>
        Promise.all(records.map(record => getFetchResult(record))))
      // Clone registration and send message.
      .then(results => {
        const registrationCopy = cloneRegistration(event.registration);
        sendMessageToDocument(
          { type: event.type, eventRegistration: registrationCopy, results })
      })
      .catch(error => {
        sendMessageToDocument(
          { type: event.type, eventRegistration:{}, results:[], error:true })
      }));
}

self.addEventListener('backgroundfetchsuccess', handleBackgroundFetchEvent);
self.addEventListener('backgroundfetchfail', handleBackgroundFetchEvent);
self.addEventListener('backgroundfetchabort', handleBackgroundFetchEvent);
