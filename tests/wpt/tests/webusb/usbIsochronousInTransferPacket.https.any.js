'use strict';

test(t => {
  let data_view = new DataView(Uint8Array.from([1, 2, 3, 4]).buffer);
  let packet = new USBIsochronousInTransferPacket('ok', data_view);
  assert_equals(packet.status, 'ok');
  assert_equals(packet.data.getInt32(0), 16909060);
}, 'Can construct a USBIsochronousInTransferPacket');

test(t => {
  let packet = new USBIsochronousInTransferPacket('stall');
  assert_equals(packet.status, 'stall');
  assert_equals(packet.data, null);

  packet = new USBIsochronousInTransferPacket('stall', null);
  assert_equals(packet.status, 'stall');
  assert_equals(packet.data, null);
}, 'Can construct a USBIsochronousInTransferPacket without a DataView');

test(t => {
  assert_throws_js(TypeError, () => {
    new USBIsochronousInTransferPacket('invalid_status');
  });
}, 'Cannot construct USBIsochronousInTransferPacket with an invalid status');

test(t => {
  assert_throws_js(TypeError, () => new USBIsochronousInTransferPacket());
}, 'Cannot construct USBIsochronousInTransferPacket without a status');
