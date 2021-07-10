// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/payment-request/

idl_test(
  ['payment-request'],
  ['dom', 'html'],
  idlArray => {
    try {
      const methods = [
        {supportedMethods: 'basic-card'},
        {supportedMethods: 'https://apple.com/apple-pay'},
      ];
      const amount = {currency: 'USD', value: '0'};
      const details = {total: {label: 'label', amount: amount} };
      window.paymentRequest = new PaymentRequest(methods, details);
    } catch (e) {
      // Surfaced below when paymentRequest is undefined.
    }

    idlArray.add_objects({
      PaymentRequest: ['paymentRequest'],
      PaymentMethodChangeEvent: ['new PaymentMethodChangeEvent("paymentmethodchange")'],
      PaymentRequestUpdateEvent: ['new PaymentRequestUpdateEvent("paymentrequestupdate")'],
      MerchantValidationEvent: ['new MerchantValidationEvent("merchantvalidation")'],
    });
  }
);
