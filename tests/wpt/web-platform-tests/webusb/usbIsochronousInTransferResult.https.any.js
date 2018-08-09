'use strict';

test(t => {
  let data_view = new DataView(Uint8Array.from([1, 2, 3, 4]).buffer);
  let packet_data_view = new DataView(data_view.buffer);
  let packets = [
      new USBIsochronousInTransferPacket('ok', packet_data_view),
      new USBIsochronousInTransferPacket('stall')
  ];

  let result = new USBIsochronousInTransferResult(packets, data_view);
  assert_equals(result.data.getInt32(0), 16909060);
  assert_equals(result.packets.length, 2);
  assert_equals(result.packets[0].status, 'ok');
  assert_equals(result.packets[0].data.getInt32(0), 16909060);
  assert_equals(result.packets[1].status, 'stall');
  assert_equals(result.packets[1].data, null);
}, 'Can construct a USBIsochronousInTransferResult');

test(t => {
  let packets = [
      new USBIsochronousInTransferPacket('stall'),
      new USBIsochronousInTransferPacket('stall')
  ];
  let result = new USBIsochronousInTransferResult(packets);
  assert_equals(result.data, null);
  assert_equals(result.packets.length, 2);
  assert_equals(result.packets[0].status, 'stall');
  assert_equals(result.packets[0].data, null);
  assert_equals(result.packets[1].status, 'stall');
  assert_equals(result.packets[1].data, null);
}, 'Can construct a USBIsochronousInTransferResult without a DataView');

test(t => {
  assert_throws(TypeError(), () => new USBIsochronousInTransferResult());
}, 'Cannot construct a USBIsochronousInTransferResult without packets');
