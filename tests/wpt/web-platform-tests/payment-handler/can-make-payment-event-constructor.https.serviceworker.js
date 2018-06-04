importScripts('/resources/testharness.js');

test(() => {
  try {
    new CanMakePaymentEvent('test', undefined);
    new CanMakePaymentEvent('test', null);
    new CanMakePaymentEvent('test', {});
  } catch (err) {
    assert_unreached(`Unexpected exception: ${err.message}`);
  }
}, 'CanMakePaymentEvent can be constructed in service worker.');

test(() => {
  const ev = new CanMakePaymentEvent('test', {
    bubbles: true,
    cancelable: true,
    composed: true,
  });
  assert_false(ev.isTrusted, 'constructed in script, so not be trusted');
  assert_true(ev.bubbles, 'set by EventInitDict');
  assert_true(ev.cancelable, 'set by EventInitDict');
  assert_true(ev.composed, 'set by EventInitDict');
  assert_equals(ev.target, null, 'initially null');
  assert_equals(ev.type, 'test');
}, 'CanMakePaymentEvent can be constructed with an EventInitDict, even if not trusted');

test(() => {
  const ev = new CanMakePaymentEvent('test', {
    topOrigin: 'https://foo.com',
    paymentRequestOrigin: 'https://bar.com',
    methodData: [],
    modifiers: [],
  });
  assert_false(ev.isTrusted, 'constructed in script, so not be trusted');
  assert_equals(ev.topOrigin, 'https://foo.com');
  assert_equals(ev.paymentRequestOrigin, 'https://bar.com');
}, 'CanMakePaymentEvent can be constructed with a CanMakePaymentEventInit, even if not trusted');

test(() => {
  const ev = new CanMakePaymentEvent('test', {});
  self.addEventListener('test', evt => {
    assert_equals(ev, evt);
  });
  self.dispatchEvent(ev);
}, 'CanMakePaymentEvent can be dispatched, even if not trusted');
