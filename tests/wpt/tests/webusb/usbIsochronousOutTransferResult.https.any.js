'use strict';

test(t => {
  let packets = [
      new USBIsochronousOutTransferPacket('ok', 42),
      new USBIsochronousOutTransferPacket('stall')
  ];

  let result = new USBIsochronousOutTransferResult(packets);
  assert_equals(result.packets.length, 2);
  assert_equals(result.packets[0].status, 'ok');
  assert_equals(result.packets[0].bytesWritten, 42);
  assert_equals(result.packets[1].status, 'stall');
  assert_equals(result.packets[1].bytesWritten, 0);
}, 'Can construct a USBIsochronousOutTransferResult');

test(t => {
  assert_throws_js(TypeError, () => new USBIsochronousOutTransferResult());
}, 'Cannot construct a USBIsochronousOutTransferResult without packets');
