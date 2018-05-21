self.addEventListener('canmakepayment', event => {
  if (event.methodData.length !== 1) {
    const msg = 'Expected exactly one method data.';
    event.respondWith(Promise.reject(new Error(msg)));
    return;
  }

  const [method] = event.methodData;
  if (!method || method.supportedMethods.length !== 1) {
    const msg = 'Expected exactly one supported method name';
    event.respondWith(Promise.reject(new Error(msg)));
    return;
  }

  if (method.data.defaultParameter !== 'defaultValue') {
    const msg = `Unexpected value for "defaultParameter": ${
      method.data.defaultParameter
    }`;
    event.respondWith(Promise.reject(new Error(msg)));
    return;
  }

  if ('defaultUnsupportedParameter' in method.data) {
    const msg = 'Unexpected "defaultUnsupportedParameter"';
    event.respondWith(Promise.reject(new Error(msg)));
    return;
  }

  if (event.modifiers.length !== 1) {
    const msg = 'Expected exactly one modifier';
    event.respondWith(Promise.reject(new Error(msg)));
    return;
  }

  const [modifier] = event.modifiers;

  if (!modifier || modifier.supportedMethods.length !== 1) {
    const msg = 'Expected exactly one supported method name in modifier';
    event.respondWith(Promise.reject(new Error(msg)));
    return;
  }

  for (const member of [
    'additionalDisplayItems',
    'modifiedUnsupportedParameter',
    'total',
  ]) {
    if (member in modifier) {
      const msg = `Unexpected member "${member}" in modifier`;
      event.respondWith(Promise.reject(new Error(msg)));
      return;
    }
  }

  const [methodName] = method.supportedMethods;
  if (methodName === 'basic-card') {
    const msg =
      '"basic-card" payment method must never be checked in CanMakePaymentEvent';
    event.respondWith(Promise.reject(new Error(msg)));
    return;
  }

  const [modifierMethodName] = modifier.supportedMethods;
  if (modifierMethodName !== methodName) {
    const msg = `Unexpected modifier method name: "${modifierMethodName}". Expected "${methodName}".`;
    event.respondWith(Promise.reject(new Error(msg)));
    return;
  }

  if (modifier.data.modifiedParameter !== 'modifiedValue') {
    const msg = `Unexpected value for 'modifiedParameter': ${
      modifier.data.modifiedParameter
    }`;
    event.respondWith(Promise.reject(new Error(msg)));
    return;
  }

  const methodAsURL = new URL(methodName);
  if (event.topOrigin !== methodAsURL.origin) {
    const msg = `Unexpected event.topOrigin: "${
      event.topOrigin
    }". Expected "${methodAsURL.origin}".`;
    event.respondWith(Promise.reject(new Error(msg)));
    return;
  }

  if (event.paymentRequestOrigin !== methodAsURL.origin) {
    const msg = `Unexpected iframe origin ${event.paymentRequestOrigin}`;
    event.respondWith(Promise.reject(new Error(msg)));
    return;
  }

  switch (methodAsURL.pathname.substr(1)) {
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
