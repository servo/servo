// This service worker script should be used by the
// protocol-handler-unregister.https.html test to detect unregistered URL
// schemes.
self.addEventListener('message', async eventInfo => {
  let success = false;
  let message = null;

  try {
    const {clientUrlMatch, navigationUrl} = eventInfo.data;
    const client = (await clients.matchAll()).find(
        client => client.url === clientUrlMatch);

    if (client) {
      try {
        await client.navigate(navigationUrl);
        success = true;
      } catch (navigateException) {
        message = `navigate failure: ${navigateException.name}`;
      }
    } else {
      message = `no client found matching ${clientUrlMatch}`;
    }
  } catch (otherException) {
    message = `other failure: ${otherException.name}`;
  }

  eventInfo.source.postMessage({success, message});
});