onmessage = e => {
  // Collect all client URLs in this origin.
  const options = { includeUncontrolled: true, type: 'all' };
  const promise = self.clients.matchAll(options)
      .then(clients => {
        const client_urls = [];
        clients.forEach(client => client_urls.push(client.url));
        e.source.postMessage(client_urls);
      });
  e.waitUntil(promise);
};
