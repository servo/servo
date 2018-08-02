'use strict';

test(t => {
  let packet = new USBIsochronousOutTransferPacket('ok', 42);
  assert_equals(packet.status, 'ok');
  assert_equals(packet.bytesWritten, 42);

  packet = new USBIsochronousOutTransferPacket('stall');
  assert_equals(packet.status, 'stall');
  assert_equals(packet.bytesWritten, 0);
}, 'Can construct USBIsochronousOutTransferPacket');

test(t => {
  assert_throws(TypeError(), () => {
    new USBIsochronousOutTransferPacket('invalid_status');
  });
}, 'Cannot construct USBIsochronousOutTransferPacket with an invalid status');

test(t => {
  assert_throws(TypeError(), () => new USBIsochronousOutTransferPacket());
}, 'Cannot construct USBIsochronousOutTransferPacket without a status');
