self.addEventListener('canmakepayment', event => {
  if (!event.currency) {
    event.respondWith(false);
    return;
  }

  if (event.currency !== 'USD') {
    event.respondWith(false);
    return;
  }

  if (!event.respondWithMinimalUI) {
    event.respondWith(false);
    return;
  }

  event.respondWithMinimalUI(event.methodData[0].data.test);
});
