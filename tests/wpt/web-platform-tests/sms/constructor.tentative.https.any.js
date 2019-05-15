// META: title=SMS Receiver API: Constructor

'use strict';

promise_test(async t => {
  let used = false;

  new SMSReceiver({
    get timeout() {
      used = true;
      return 60;
    }
  });

  assert_true(used, 'constructor options "timeout" member was used');
}, 'constructor uses timeout property');

promise_test(async t => {
  try {
    new SMSReceiver({timeout: 0});
    assert_unreached('Timeout 0 should reject');
  } catch (error) {
    assert_equals(error.name, 'TypeError');
  }
}, 'constructor throws with invalid timeout (0)');

promise_test(async t => {
  try {
    new SMSReceiver({timeout: null});
    assert_unreached('Timeout of null should reject');
  } catch (error) {
    assert_equals(error.name, 'TypeError');
  }
}, 'constructor throws with invalid timeout (null)');

promise_test(async t => {
  try {
    new SMSReceiver({timeout: -1});
    assert_unreached('Timeout negative numbers should reject');
  } catch (error) {
    assert_equals(error.name, 'TypeError');
  }
}, 'constructor throws with invalid timeout (-1)');

promise_test(async t => {
  try {
    new SMSReceiver({timeout: NaN});
    assert_unreached('Timeout of NaN should reject');
  } catch (error) {
    assert_equals(error.name, 'TypeError');
  }
}, 'constructor throws with invalid timeout (NaN)');

promise_test(async t => {
  new SMSReceiver();
}, 'constructor uses a default value for the timeout when none is passed');

promise_test(async t => {
  new SMSReceiver({timeout: undefined});
}, 'constructor uses a default value for the timeout');
