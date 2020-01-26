// META: script=/webusb/resources/fake-devices.js
// META: script=/webusb/resources/usb-helpers.js
'use strict';

test(t => {
  let data_view = new DataView(Uint8Array.from([1, 2, 3, 4]).buffer);
  let result = new USBInTransferResult('ok', data_view);
  assert_equals(result.status, 'ok');
  assert_equals(result.data.getInt32(0), 16909060);
}, 'Can construct a USBInTransferResult');

test(t => {
  let result = new USBInTransferResult('stall');
  assert_equals(result.status, 'stall');
  assert_equals(result.data, null);

  result = new USBInTransferResult('babble', null);
  assert_equals(result.status, 'babble');
  assert_equals(result.data, null);
}, 'Can construct a USBInTransferResult without a DataView');

test(t => {
  assert_throws_js(TypeError, () => new USBInTransferResult('invalid_status'));
}, 'Cannot construct USBInTransferResult with an invalid status');

test(t => {
  assert_throws_js(TypeError, () => new USBInTransferResult());
}, 'Cannot construct USBInTransferResult without a status');
