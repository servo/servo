importScripts('/resources/testharness.js');

test(() => {
  assert_false('PaymentRequestEvent' in self);
}, 'PaymentRequestEvent constructor must not be exposed in worker');

done();
