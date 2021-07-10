// This worker reports back the final state of FetchEvent.handled (RESOLVED or
// REJECTED) to the test.

// Send a message to the client with the client id.
function send_message_to_client(message, clientId) {
  clients.get(clientId).then((client) => {
    client.postMessage(message);
  });
}

self.addEventListener('fetch', function(event) {
  const clientId = (event.request.mode === 'navigate') ?
      event.resultingClientId : event.clientId;

  try {
    event.handled.then(() => {
      send_message_to_client('RESOLVED', clientId);
    }, () => {
      send_message_to_client('REJECTED', clientId);
    });
  } catch (e) {
    send_message_to_client('FAILED', clientId);
    return;
  }

  const search = new URL(event.request.url).search;
  switch (search) {
    case '?respondWith-not-called':
      break;
    case '?respondWith-not-called-and-event-canceled':
      event.preventDefault();
      break;
    case '?respondWith-called-and-promise-resolved':
      event.respondWith(Promise.resolve(new Response('body')));
      break;
    case '?respondWith-called-and-promise-resolved-to-invalid-response':
      event.respondWith(Promise.resolve('invalid response'));
      break;
    case '?respondWith-called-and-promise-rejected':
      event.respondWith(Promise.reject(new Error('respondWith rejected')));
      break;
  }
});
