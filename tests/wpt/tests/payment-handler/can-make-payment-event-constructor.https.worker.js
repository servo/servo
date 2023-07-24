importScripts('/resources/testharness.js');

test(() => {
  assert_false('CanMakePaymentEvent' in self);
}, 'CanMakePaymentEvent constructor must not be exposed in worker');

done();
