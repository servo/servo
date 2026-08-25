let responseType = 'canMakePayment-true';

self.addEventListener('canmakepayment', event => {
  if (event.methodData) {
    const msg = 'Expected no method data.';
    event.respondWith(Promise.reject(new Error(msg)));
    return;
  }

  if (event.modifiers) {
    const msg = 'Expected no modifiers';
    event.respondWith(Promise.reject(new Error(msg)));
    return;
  }

  if (event.topOrigin) {
    const msg = `Unexpected topOrigin.`;
    event.respondWith(Promise.reject(new Error(msg)));
    return;
  }

  if (event.paymentRequestOrigin) {
    const msg = `Unexpected iframe origin.`;
    event.respondWith(Promise.reject(new Error(msg)));
    return;
  }

  switch (responseType) {
    case 'canMakePayment-true':
      event.respondWith(true);
      break;
    case 'canMakePayment-false':
      event.respondWith(false);
      break;
    case 'canMakePayment-promise-true':
      event.respondWith(Promise.resolve(true));
      break;
    case 'canMakePayment-promise-false':
      event.respondWith(Promise.resolve(false));
      break;
    case 'canMakePayment-custom-error':
      event.respondWith(Promise.reject(new Error('Custom error')));
      break;
    default:
      const msg = `Unrecognized payment method name "${methodName}".`;
      event.respondWith(Promise.reject(new Error(msg)));
      break;
  }
});

self.addEventListener('paymentrequest', event => {
  responseType = event.methodData[0].data.responseType;
  event.respondWith({
    methodName: event.methodData[0].supportedMethods,
    details: {status: 'success'},
  });
});
