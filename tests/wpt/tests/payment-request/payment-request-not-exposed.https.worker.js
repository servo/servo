importScripts("/resources/testharness.js");

test(() => {
  assert_true(isSecureContext);
  assert_false('PaymentRequest' in self);
}, "PaymentRequest constructor must not be exposed in worker global scope");
done();
