self.addEventListener('canmakepayment', event => {
  event.respondWith(true);
});

self.addEventListener('paymentrequest', event => {
  event.respondWith({
    methodName: event.methodData[0].supportedMethods,
    details: {status: 'success'},
  });
});
