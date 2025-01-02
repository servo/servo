'use strict';

test(t => {
  let result = new USBOutTransferResult('ok', 42);
  assert_equals(result.status, 'ok');
  assert_equals(result.bytesWritten, 42);

  result = new USBOutTransferResult('stall');
  assert_equals(result.status, 'stall');
  assert_equals(result.bytesWritten, 0);
}, 'Can construct USBOutTransferResult');

test(t => {
  assert_throws_js(TypeError, () => new USBOutTransferResult('invalid_status'));
}, 'Cannot construct USBOutTransferResult with an invalid status');

test(t => {
  assert_throws_js(TypeError, () => new USBOutTransferResult());
}, 'Cannot construct USBOutTransferResult without a status');
