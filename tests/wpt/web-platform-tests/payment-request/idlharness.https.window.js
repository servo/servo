// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/payment-request/

idl_test(
  ['payment-request'],
  ['dom', 'html'],
  idlArray => {
    try {
      const methods = [{supportedMethods: 'foo'}];
      const amount = {currency: 'USD', value: '0'};
      const details = {total: {label: 'bar', amount: amount} };
      window.paymentRequest = new PaymentRequest(methods, details);
    } catch (e) {
      // Will be surfaced in idlharness.js's test_object below.
    }

    idlArray.add_objects({
      PaymentRequest: ['paymentRequest'],
    });
  },
  'Setup for Payment Request API IDL tests.'
);
