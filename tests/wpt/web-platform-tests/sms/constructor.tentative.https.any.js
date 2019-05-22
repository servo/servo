// META: title=SMS Receiver API: Constructor

'use strict';

test(function() {
  let used = false;

  new SMSReceiver({
    get timeout() {
      used = true;
      return 60;
    }
  });

  assert_true(used, 'constructor options "timeout" member was used');
}, 'constructor uses timeout property');

test(function() {
  assert_throws(new TypeError(), function () {
    new SMSReceiver({timeout: 0});
    assert_unreached('Timeout 0 should reject');
  });
}, 'constructor throws with invalid timeout (0)');

test(function() {
  assert_throws(new TypeError(), function () {
    new SMSReceiver({timeout: null});
    assert_unreached('Timeout of null should reject');
  });
}, 'constructor throws with invalid timeout (null)');

test(function() {
  assert_throws(new TypeError(), function () {
    new SMSReceiver({timeout: -1});
    assert_unreached('Timeout negative numbers should reject');
  });
}, 'constructor throws with invalid timeout (-1)');

test(function() {
  assert_throws(new TypeError(), function () {
    new SMSReceiver({timeout: NaN});
    assert_unreached('Timeout of NaN should reject');
  });
}, 'constructor throws with invalid timeout (NaN)');

test(function() {
  new SMSReceiver();
}, 'constructor uses a default value for the timeout when none is passed');

test(function() {
  new SMSReceiver({timeout: undefined});
}, 'constructor uses a default value for the timeout');
