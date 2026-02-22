importScripts('/resources/testharness.js');

test(() => {
  try {
    new PaymentRequestEvent('test', undefined);
    new PaymentRequestEvent('test', null);
    new PaymentRequestEvent('test', {});
  } catch (err) {
    assert_unreached(`Unexpected exception: ${err.message}`);
  }
}, 'PaymentRequestEvent can be constucted in service worker.');

test(() => {
  const ev = new PaymentRequestEvent('test', {
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
}, 'PaymentRequestEvent can be constructed with an EventInitDict, even if not trusted');

test(() => {
  const ev = new PaymentRequestEvent('test', {
    topOrigin: 'https://foo.com',
    paymentRequestOrigin: 'https://bar.com',
    methodData: [],
    modifiers: [],
  });
  assert_false(ev.isTrusted, 'constructed in script, so not be trusted');
  assert_equals(ev.topOrigin, 'https://foo.com');
  assert_equals(ev.paymentRequestOrigin, 'https://bar.com');
}, 'PaymentRequestEvent can be constructed with a PaymentRequestEventInit, even if not trusted');

test(() => {
  const ev = new PaymentRequestEvent('test', {});
  self.addEventListener('test', evt => {
    assert_equals(ev, evt);
  });
  self.dispatchEvent(ev);
}, 'PaymentRequestEvent can be dispatched, even if not trusted');
