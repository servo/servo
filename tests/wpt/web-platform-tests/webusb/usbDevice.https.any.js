// META: script=/webusb/resources/fake-devices.js
// META: script=/webusb/resources/usb-helpers.js
// META: global=sharedworker
'use strict';

function assertRejectsWithNotFoundError(promise) {
  return assertRejectsWithError(promise, 'NotFoundError');
}

function assertRejectsWithNotOpenError(promise) {
  return assertRejectsWithError(
      promise, 'InvalidStateError', 'The device must be opened first.');
}

function assertRejectsWithNotConfiguredError(promise) {
  return assertRejectsWithError(
      promise, 'InvalidStateError',
      'The device must have a configuration selected.');
}

usb_test(() => {
  return getFakeDevice().then(({ device, fakeDevice }) => {
    return waitForDisconnect(fakeDevice)
      .then(() => assertRejectsWithNotFoundError(device.open()));
  });
}, 'open rejects when called on a disconnected device');

usb_test(() => {
  return getFakeDevice().then(({ device, fakeDevice }) => {
    return device.open()
      .then(() => waitForDisconnect(fakeDevice))
      .then(() => {
        assert_false(device.opened);
      });
  });
}, 'disconnection closes the device');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    assert_false(device.opened);
    return device.open().then(() => {
      assert_true(device.opened);
      return device.close().then(() => {
        assert_false(device.opened);
      });
    });
  });
}, 'a device can be opened and closed');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    return device.open()
      .then(() => device.open())
      .then(() => device.open())
      .then(() => device.open())
      .then(() => device.close())
      .then(() => device.close())
      .then(() => device.close())
      .then(() => device.close());
  });
}, 'open and close can be called multiple times');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    const message =
        'An operation that changes the device state is in progress.';
    return Promise.all([
        device.open(),
        assertRejectsWithError(device.open(), 'InvalidStateError', message),
        assertRejectsWithError(device.close(), 'InvalidStateError', message),
    ]).then(() => Promise.all([
        device.close(),
        assertRejectsWithError(device.open(), 'InvalidStateError', message),
        assertRejectsWithError(device.close(), 'InvalidStateError', message),
    ]));
  });
}, 'open and close cannot be called again while open or close are in progress');

usb_test(() => {
  return getFakeDevice().then(({ device, fakeDevice }) => {
    return device.open()
      .then(() => waitForDisconnect(fakeDevice))
      .then(() => assertRejectsWithNotFoundError(device.close()));
  });
}, 'close rejects when called on a disconnected device');

usb_test(() => {
  return getFakeDevice().then(({ device, fakeDevice }) => {
    return device.open()
      .then(() => waitForDisconnect(fakeDevice))
      .then(() => assertRejectsWithNotFoundError(device.selectConfiguration(1)));
  });
}, 'selectConfiguration rejects when called on a disconnected device');

usb_test(() => {
  return getFakeDevice().then(({ device }) => Promise.all([
      assertRejectsWithNotOpenError(device.selectConfiguration(1)),
      assertRejectsWithNotOpenError(device.claimInterface(0)),
      assertRejectsWithNotOpenError(device.releaseInterface(0)),
      assertRejectsWithNotOpenError(device.selectAlternateInterface(0, 1)),
      assertRejectsWithNotOpenError(device.controlTransferIn({
          requestType: 'vendor',
          recipient: 'device',
          request: 0x42,
          value: 0x1234,
          index: 0x5678
      }, 7)),
      assertRejectsWithNotOpenError(device.controlTransferOut({
          requestType: 'vendor',
          recipient: 'device',
          request: 0x42,
          value: 0x1234,
          index: 0x5678
      }, new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]))),
      assertRejectsWithNotOpenError(device.clearHalt('in', 1)),
      assertRejectsWithNotOpenError(device.transferIn(1, 8)),
      assertRejectsWithNotOpenError(
          device.transferOut(1, new ArrayBuffer(8))),
      assertRejectsWithNotOpenError(device.isochronousTransferIn(1, [8])),
      assertRejectsWithNotOpenError(
          device.isochronousTransferOut(1, new ArrayBuffer(8), [8])),
      assertRejectsWithNotOpenError(device.reset())
  ]));
}, 'methods requiring it reject when the device is not open');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    assert_equals(device.configuration, null);
    return device.open()
      .then(() => {
        assert_equals(device.configuration, null);
        return device.selectConfiguration(1);
      })
      .then(() => {
        assertDeviceInfoEquals(
            device.configuration, fakeDeviceInit.configurations[0]);
      })
      .then(() => device.close());
  });
}, 'device configuration can be set and queried');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    assert_equals(device.configuration, null);
    return device.open()
      .then(() => assertRejectsWithError(
            device.selectConfiguration(3), 'NotFoundError',
            'The configuration value provided is not supported by the device.'))
      .then(() => device.close());
  });
}, 'selectConfiguration rejects on invalid configurations');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    assert_equals(device.configuration, null);
    return device.open().then(() => Promise.all([
        assertRejectsWithNotConfiguredError(device.claimInterface(0)),
        assertRejectsWithNotConfiguredError(device.releaseInterface(0)),
        assertRejectsWithNotConfiguredError(device.selectAlternateInterface(0, 1)),
        assertRejectsWithNotConfiguredError(device.controlTransferIn({
            requestType: 'vendor',
            recipient: 'device',
            request: 0x42,
            value: 0x1234,
            index: 0x5678
        }, 7)),
        assertRejectsWithNotConfiguredError(device.controlTransferOut({
            requestType: 'vendor',
            recipient: 'device',
            request: 0x42,
            value: 0x1234,
            index: 0x5678
        }, new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]))),
        assertRejectsWithNotConfiguredError(device.clearHalt('in', 1)),
        assertRejectsWithNotConfiguredError(device.transferIn(1, 8)),
        assertRejectsWithNotConfiguredError(
            device.transferOut(1, new ArrayBuffer(8))),
        assertRejectsWithNotConfiguredError(
            device.isochronousTransferIn(1, [8])),
        assertRejectsWithNotConfiguredError(
            device.isochronousTransferOut(1, new ArrayBuffer(8), [8])),
    ])).then(() => device.close());
  });
}, 'methods requiring it reject when the device is unconfigured');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => device.claimInterface(0))
      .then(() => {
        assert_true(device.configuration.interfaces[0].claimed);
        return device.releaseInterface(0);
      })
      .then(() => {
        assert_false(device.configuration.interfaces[0].claimed);
        return device.close();
      });
  });
}, 'an interface can be claimed and released');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => device.claimInterface(0))
      .then(() => {
        assert_true(device.configuration.interfaces[0].claimed);
        return device.close(0);
      })
      .then(() => {
        assert_false(device.configuration.interfaces[0].claimed);
      });
  });
}, 'interfaces are released on close');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    const message = 'The interface number provided is not supported by the ' +
                    'device in its current configuration.';
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => Promise.all([
          assertRejectsWithError(
              device.claimInterface(2), 'NotFoundError', message),
          assertRejectsWithError(
              device.releaseInterface(2), 'NotFoundError', message),
      ]))
      .then(() => device.close());
  });
}, 'a non-existent interface cannot be claimed or released');

usb_test(() => {
  return getFakeDevice().then(({ device, fakeDevice }) => {
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => waitForDisconnect(fakeDevice))
      .then(() => assertRejectsWithNotFoundError(device.claimInterface(0)));
  });
}, 'claimInterface rejects when called on a disconnected device');

usb_test(() => {
  return getFakeDevice().then(({ device, fakeDevice }) => {
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => device.claimInterface(0))
      .then(() => waitForDisconnect(fakeDevice))
      .then(() => assertRejectsWithNotFoundError(device.releaseInterface(0)));
  });
}, 'releaseInterface rejects when called on a disconnected device');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    return device.open()
      .then(() => device.selectConfiguration(2))
      .then(() => device.claimInterface(0))
      .then(() => device.selectAlternateInterface(0, 1))
      .then(() => device.close());
  });
}, 'can select an alternate interface');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    return device.open()
      .then(() => device.selectConfiguration(2))
      .then(() => device.claimInterface(0))
      .then(() => assertRejectsWithError(
          device.selectAlternateInterface(0, 2), 'NotFoundError',
          'The alternate setting provided is not supported by the device in ' +
          'its current configuration.'))
      .then(() => device.close());
  });
}, 'cannot select a non-existent alternate interface');

usb_test(() => {
  return getFakeDevice().then(({ device, fakeDevice }) => {
    return device.open()
      .then(() => device.selectConfiguration(2))
      .then(() => device.claimInterface(0))
      .then(() => waitForDisconnect(fakeDevice))
      .then(() => assertRejectsWithNotFoundError(device.selectAlternateInterface(0, 1)));
  });
}, 'selectAlternateInterface rejects when called on a disconnected device');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => device.controlTransferIn({
        requestType: 'vendor',
        recipient: 'device',
        request: 0x42,
        value: 0x1234,
        index: 0x5678
      }, 7))
      .then(result => {
        assert_true(result instanceof USBInTransferResult);
        assert_equals(result.status, 'ok');
        assert_equals(result.data.byteLength, 7);
        assert_equals(result.data.getUint16(0), 0x07);
        assert_equals(result.data.getUint8(2), 0x42);
        assert_equals(result.data.getUint16(3), 0x1234);
        assert_equals(result.data.getUint16(5), 0x5678);
        return device.close();
      });
  });
}, 'can issue IN control transfer');

usb_test(() => {
  return getFakeDevice().then(({ device, fakeDevice }) => {
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => waitForDisconnect(fakeDevice))
      .then(() => assertRejectsWithNotFoundError(device.controlTransferIn({
          requestType: 'vendor',
          recipient: 'device',
          request: 0x42,
          value: 0x1234,
          index: 0x5678
        }, 7)));
  });
}, 'controlTransferIn rejects when called on a disconnected device');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => device.controlTransferOut({
        requestType: 'vendor',
        recipient: 'device',
        request: 0x42,
        value: 0x1234,
        index: 0x5678
      }, new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8])))
    .then(result => {
      assert_true(result instanceof USBOutTransferResult);
      assert_equals(result.status, 'ok');
      assert_equals(result.bytesWritten, 8);
      return device.close();
    });
  });
}, 'can issue OUT control transfer');

usb_test(() => {
  return getFakeDevice().then(({ device, fakeDevice }) => {
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => waitForDisconnect(fakeDevice))
      .then(() => assertRejectsWithNotFoundError(device.controlTransferOut({
          requestType: 'vendor',
          recipient: 'device',
          request: 0x42,
          value: 0x1234,
          index: 0x5678
        }, new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]))));
  });
}, 'controlTransferOut rejects when called on a disconnected device');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    let interfaceRequest = {
        requestType: 'vendor',
        recipient: 'interface',
        request: 0x42,
        value: 0x1234,
        index: 0x5600  // Last byte of index is interface number.
    };
    let endpointRequest = {
        requestType: 'vendor',
        recipient: 'endpoint',
        request: 0x42,
        value: 0x1234,
        index: 0x5681  // Last byte of index is endpoint address.
    };
    let data = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => Promise.all([
          assertRejectsWithError(
              device.controlTransferIn(interfaceRequest, 7),
              'InvalidStateError'),
          assertRejectsWithError(
              device.controlTransferIn(endpointRequest, 7),
              'NotFoundError'),
          assertRejectsWithError(
              device.controlTransferOut(interfaceRequest, data),
              'InvalidStateError'),
          assertRejectsWithError(
              device.controlTransferOut(endpointRequest, data),
              'NotFoundError'),
      ]))
      .then(() => device.claimInterface(0))
      .then(() => Promise.all([
          device.controlTransferIn(interfaceRequest, 7).then(result => {
            assert_true(result instanceof USBInTransferResult);
            assert_equals(result.status, 'ok');
            assert_equals(result.data.byteLength, 7);
            assert_equals(result.data.getUint16(0), 0x07);
            assert_equals(result.data.getUint8(2), 0x42);
            assert_equals(result.data.getUint16(3), 0x1234);
            assert_equals(result.data.getUint16(5), 0x5600);
          }),
          device.controlTransferIn(endpointRequest, 7).then(result => {
            assert_true(result instanceof USBInTransferResult);
            assert_equals(result.status, 'ok');
            assert_equals(result.data.byteLength, 7);
            assert_equals(result.data.getUint16(0), 0x07);
            assert_equals(result.data.getUint8(2), 0x42);
            assert_equals(result.data.getUint16(3), 0x1234);
            assert_equals(result.data.getUint16(5), 0x5681);
          }),
          device.controlTransferOut(interfaceRequest, data),
          device.controlTransferOut(endpointRequest, data),
      ]))
      .then(() => device.close());
  });
}, 'requests to interfaces and endpoint require an interface claim');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => device.claimInterface(0))
      .then(() => device.clearHalt('in', 1))
      .then(() => device.close());
  });
}, 'can clear a halt condition');

usb_test(() => {
  return getFakeDevice().then(({ device, fakeDevice }) => {
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => device.claimInterface(0))
      .then(() => waitForDisconnect(fakeDevice))
      .then(() => assertRejectsWithNotFoundError(device.clearHalt('in', 1)));
  });
}, 'clearHalt rejects when called on a disconnected device');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    let data = new DataView(new ArrayBuffer(1024));
    for (let i = 0; i < 1024; ++i)
      data.setUint8(i, i & 0xff);
    const notFoundMessage = 'The specified endpoint is not part of a claimed ' +
                            'and selected alternate interface.';
    const rangeError = 'The specified endpoint number is out of range.';
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => device.claimInterface(0))
      .then(() => Promise.all([
          assertRejectsWithError(device.transferIn(2, 8),
                                 'NotFoundError', notFoundMessage), // Unclaimed
          assertRejectsWithError(device.transferIn(3, 8), 'NotFoundError',
                                 notFoundMessage), // Non-existent
          assertRejectsWithError(
              device.transferIn(16, 8), 'IndexSizeError', rangeError),
          assertRejectsWithError(device.transferOut(2, data),
                                 'NotFoundError', notFoundMessage), // Unclaimed
          assertRejectsWithError(device.transferOut(3, data), 'NotFoundError',
                                 notFoundMessage), // Non-existent
          assertRejectsWithError(
              device.transferOut(16, data), 'IndexSizeError', rangeError),
      ]));
  });
}, 'transfers to unavailable endpoints are rejected');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => device.claimInterface(0))
      .then(() => device.transferIn(1, 8))
      .then(result => {
        assert_true(result instanceof USBInTransferResult);
        assert_equals(result.status, 'ok');
        assert_equals(result.data.byteLength, 8);
        for (let i = 0; i < 8; ++i)
          assert_equals(result.data.getUint8(i), i, 'mismatch at byte ' + i);
        return device.close();
      });
  });
}, 'can issue IN interrupt transfer');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => device.claimInterface(1))
      .then(() => device.transferIn(2, 1024))
      .then(result => {
        assert_true(result instanceof USBInTransferResult);
        assert_equals(result.status, 'ok');
        assert_equals(result.data.byteLength, 1024);
        for (let i = 0; i < 1024; ++i)
          assert_equals(result.data.getUint8(i), i & 0xff,
                        'mismatch at byte ' + i);
        return device.close();
      });
  });
}, 'can issue IN bulk transfer');

usb_test(() => {
  return getFakeDevice().then(({ device, fakeDevice }) => {
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => device.claimInterface(1))
      .then(() => waitForDisconnect(fakeDevice))
      .then(() => assertRejectsWithNotFoundError(device.transferIn(2, 1024)));
  });
}, 'transferIn rejects if called on a disconnected device');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => device.claimInterface(1))
      .then(() => {
        let data = new DataView(new ArrayBuffer(1024));
        for (let i = 0; i < 1024; ++i)
          data.setUint8(i, i & 0xff);
        return device.transferOut(2, data);
      })
      .then(result => {
        assert_true(result instanceof USBOutTransferResult);
        assert_equals(result.status, 'ok');
        assert_equals(result.bytesWritten, 1024);
        return device.close();
      });
  });
}, 'can issue OUT bulk transfer');

usb_test(() => {
  return getFakeDevice().then(({ device, fakeDevice }) => {
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => device.claimInterface(1))
      .then(() => {
        let data = new DataView(new ArrayBuffer(1024));
        for (let i = 0; i < 1024; ++i)
          data.setUint8(i, i & 0xff);
        return waitForDisconnect(fakeDevice)
          .then(() => assertRejectsWithNotFoundError(device.transferOut(2, data)));
      });
  });
}, 'transferOut rejects if called on a disconnected device');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    return device.open()
      .then(() => device.selectConfiguration(2))
      .then(() => device.claimInterface(0))
      .then(() => device.selectAlternateInterface(0, 1))
      .then(() => device.isochronousTransferIn(
          1, [64, 64, 64, 64, 64, 64, 64, 64]))
      .then(result => {
        assert_true(result instanceof USBIsochronousInTransferResult);
        assert_equals(result.data.byteLength, 64 * 8, 'buffer size');
        assert_equals(result.packets.length, 8, 'number of packets');
        let byteOffset = 0;
        for (let i = 0; i < result.packets.length; ++i) {
          assert_true(
              result.packets[i] instanceof USBIsochronousInTransferPacket);
          assert_equals(result.packets[i].status, 'ok');
          assert_equals(result.packets[i].data.byteLength, 64);
          assert_equals(result.packets[i].data.buffer, result.data.buffer);
          assert_equals(result.packets[i].data.byteOffset, byteOffset);
          for (let j = 0; j < 64; ++j)
            assert_equals(result.packets[i].data.getUint8(j), j & 0xff,
                          'mismatch at byte ' + j + ' of packet ' + i);
          byteOffset += result.packets[i].data.byteLength;
        }
        return device.close();
      });
  });
}, 'can issue IN isochronous transfer');

usb_test(() => {
  return getFakeDevice().then(({ device, fakeDevice }) => {
    return device.open()
      .then(() => device.selectConfiguration(2))
      .then(() => device.claimInterface(0))
      .then(() => device.selectAlternateInterface(0, 1))
      .then(() => waitForDisconnect(fakeDevice))
      .then(() => assertRejectsWithNotFoundError(device.isochronousTransferIn(
          1, [64, 64, 64, 64, 64, 64, 64, 64])));
  });
}, 'isochronousTransferIn rejects when called on a disconnected device');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    return device.open()
      .then(() => device.selectConfiguration(2))
      .then(() => device.claimInterface(0))
      .then(() => device.selectAlternateInterface(0, 1))
      .then(() => {
        let data = new DataView(new ArrayBuffer(64 * 8));
        for (let i = 0; i < 8; ++i) {
          for (let j = 0; j < 64; ++j)
            data.setUint8(i * j, j & 0xff);
        }
        return device.isochronousTransferOut(
            1, data, [64, 64, 64, 64, 64, 64, 64, 64]);
      })
      .then(result => {
        assert_true(result instanceof USBIsochronousOutTransferResult);
        assert_equals(result.packets.length, 8, 'number of packets');
        let byteOffset = 0;
        for (let i = 0; i < result.packets.length; ++i) {
          assert_true(
              result.packets[i] instanceof USBIsochronousOutTransferPacket);
          assert_equals(result.packets[i].status, 'ok');
          assert_equals(result.packets[i].bytesWritten, 64);
        }
        return device.close();
      });
  });
}, 'can issue OUT isochronous transfer');

usb_test(() => {
  return getFakeDevice().then(({ device, fakeDevice }) => {
    return device.open()
      .then(() => device.selectConfiguration(2))
      .then(() => device.claimInterface(0))
      .then(() => device.selectAlternateInterface(0, 1))
      .then(() => {
        let data = new DataView(new ArrayBuffer(64 * 8));
        for (let i = 0; i < 8; ++i) {
          for (let j = 0; j < 64; ++j)
            data.setUint8(i * j, j & 0xff);
        }
        return waitForDisconnect(fakeDevice)
          .then(() => assertRejectsWithNotFoundError(device.isochronousTransferOut(
              1, data, [64, 64, 64, 64, 64, 64, 64, 64])));
      });
  });
}, 'isochronousTransferOut rejects when called on a disconnected device');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    return device.open().then(() => device.reset()).then(() => device.close());
  });
}, 'can reset the device');

usb_test(() => {
  return getFakeDevice().then(({ device, fakeDevice }) => {
    return device.open()
      .then(() => waitForDisconnect(fakeDevice))
      .then(() => assertRejectsWithNotFoundError(device.reset()));
  });
}, 'resetDevice rejects when called on a disconnected device');
