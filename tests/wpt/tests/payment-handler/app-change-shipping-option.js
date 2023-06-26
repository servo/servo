self.addEventListener('canmakepayment', (event) => {
  event.respondWith(true);
});

async function responder(event) {
  const methodName = event.methodData[0].supportedMethods;
  const shippingOption = event.shippingOptions[0].id;
  const shippingAddress = {
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
  };
  if (!event.changeShippingOption) {
    return {
      methodName,
      details: {
        changeShippingOptionReturned:
          'The changeShippingOption() method is not implemented.',
      },
    };
  }
  let changeShippingOptionReturned;
  try {
    const response = await event.changeShippingOption(shippingOption);
    changeShippingOptionReturned = response;
  } catch (err) {
    changeShippingOptionReturned = err.message;
  }
  return {methodName, details: {changeShippingOptionReturned}, shippingAddress,
      shippingOption};
}

self.addEventListener('paymentrequest', (event) => {
  event.respondWith(responder(event));
});
