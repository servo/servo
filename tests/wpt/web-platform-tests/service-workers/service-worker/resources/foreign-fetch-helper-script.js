function handle_message(e) {
  fetch(e.data.url)
    .then(response => response.text())
    .then(text => e.ports[0].postMessage('Success: ' + text))
    .catch(error => e.ports[0].postMessage('Error: ' + error));
}

self.onmessage = handle_message;
self.onconnect = e => {
  e.ports[0].onmessage = handle_message;
};
