self.addEventListener('canmakepayment', (event) => {
  event.respondWith(true);
});

function responder(event) {
  const methodName = event.methodData[0].supportedMethods;
  const shippingOption = event.paymentOptions.requestShipping
                            ? event.shippingOptions[0].id
                            : '';
  const payerName =
      event.paymentOptions.requestPayerName ? 'John Smith' : '';
  const payerEmail =
      event.paymentOptions.requestPayerEmail ? 'smith@gmail.com' : '';
  const payerPhone =
      event.paymentOptions.requestPayerPhone ? '+15555555555' : '';
  const shippingAddress = event.paymentOptions.requestShipping ? {
    addressLine: [
      '1875 Explorer St #1000',
    ],
    city: 'Reston',
    country: 'US',
    dependentLocality: '',
    organization: 'Google',
    phone: '+15555555555',
    postalCode: '20190',
    recipient: 'John Smith',
    region: 'VA',
    sortingCode: '',
  } : {};

  return {
    methodName,
    details: { token: '123456789'},
    payerName,
    payerEmail,
    payerPhone,
    shippingAddress,
    shippingOption
  };
}

self.addEventListener('paymentrequest', (event) => {
  event.respondWith(responder(event));
});
