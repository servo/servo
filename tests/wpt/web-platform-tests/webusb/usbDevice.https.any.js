// META: timeout=long
// META: script=/webusb/resources/fake-devices.js
// META: script=/webusb/resources/usb-helpers.js
'use strict';

function assertRejectsWithNotFoundError(promise) {
  return assertRejectsWithError(promise, 'NotFoundError');
}

function assertRejectsWithTypeError(promise) {
  return assertRejectsWithError(promise, 'TypeError');
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

function assertRejectsWithDeviceStateChangeInProgressError(promise) {
  return assertRejectsWithError(
    promise, 'InvalidStateError',
    'An operation that changes the device state is in progress.');
}

function assertRejectsWithInterfaceStateChangeInProgressError(promise) {
  return assertRejectsWithError(
    promise, 'InvalidStateError',
    'An operation that changes interface state is in progress.');
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

usb_test(async () => {
  let { device } = await getFakeDevice();
  await Promise.all([
    device.open(),
    assertRejectsWithDeviceStateChangeInProgressError(device.open()),
    assertRejectsWithDeviceStateChangeInProgressError(device.close()),
  ]);
  await Promise.all([
    device.close(),
    assertRejectsWithDeviceStateChangeInProgressError(device.open()),
    assertRejectsWithDeviceStateChangeInProgressError(device.close()),
  ]);
}, 'open and close cannot be called again while open or close are in progress');

usb_test(async () => {
  let { device } = await getFakeDevice();
  await device.open();
  return Promise.all([
    device.selectConfiguration(1),
    assertRejectsWithDeviceStateChangeInProgressError(
        device.claimInterface(0)),
    assertRejectsWithDeviceStateChangeInProgressError(
        device.releaseInterface(0)),
    assertRejectsWithDeviceStateChangeInProgressError(device.open()),
    assertRejectsWithDeviceStateChangeInProgressError(
        device.selectConfiguration(1)),
    assertRejectsWithDeviceStateChangeInProgressError(device.reset()),
    assertRejectsWithDeviceStateChangeInProgressError(
        device.selectAlternateInterface(0, 0)),
    assertRejectsWithDeviceStateChangeInProgressError(
        device.controlTransferOut({
          requestType: 'standard',
          recipient: 'interface',
          request: 0x42,
          value: 0x1234,
          index: 0x0000,
        })),
    assertRejectsWithDeviceStateChangeInProgressError(
        device.controlTransferOut({
          requestType: 'standard',
          recipient: 'interface',
          request: 0x42,
          value: 0x1234,
          index: 0x0000,
        }, new Uint8Array([1, 2, 3]))),
    assertRejectsWithDeviceStateChangeInProgressError(
        device.controlTransferIn({
          requestType: 'standard',
          recipient: 'interface',
          request: 0x42,
          value: 0x1234,
          index: 0x0000
        }, 0)),
    assertRejectsWithDeviceStateChangeInProgressError(device.close()),
  ]);
}, 'device operations reject if an device state change is in progress');

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

usb_test(async () => {
  let { device } = await getFakeDevice();
  assert_equals(device.configuration, null);
  await device.open();
  assert_equals(device.configuration, null);
  await device.selectConfiguration(1);
  await device.selectConfiguration(1);
  assertDeviceInfoEquals(
      device.configuration, fakeDeviceInit.configurations[0]);
  await device.selectConfiguration(2);
  assertDeviceInfoEquals(
      device.configuration, fakeDeviceInit.configurations[1]);
  await device.close();
}, 'a device configuration value can be set again');

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

usb_test(async () => {
  let { device } = await getFakeDevice()
  await device.open();
  await device.selectConfiguration(1);
  await device.claimInterface(0);
  assert_true(device.configuration.interfaces[0].claimed);
  await device.claimInterface(0);
  assert_true(device.configuration.interfaces[0].claimed);
  await device.close();
}, 'an interface can be claimed multiple times');

usb_test(async () => {
  let { device } = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(1);
  await device.claimInterface(0);
  assert_true(device.configuration.interfaces[0].claimed);
  await device.releaseInterface(0);
  assert_false(device.configuration.interfaces[0].claimed);
  await device.releaseInterface(0);
  assert_false(device.configuration.interfaces[0].claimed);
  await device.close();
}, 'an interface can be released multiple times');

usb_test(async () => {
  let { device } = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(1);
  return Promise.all([
    device.claimInterface(0),
    assertRejectsWithInterfaceStateChangeInProgressError(
        device.claimInterface(0)),
    assertRejectsWithInterfaceStateChangeInProgressError(
        device.releaseInterface(0)),
    assertRejectsWithInterfaceStateChangeInProgressError(device.open()),
    assertRejectsWithInterfaceStateChangeInProgressError(
        device.selectConfiguration(1)),
    assertRejectsWithInterfaceStateChangeInProgressError(device.reset()),
    assertRejectsWithInterfaceStateChangeInProgressError(
        device.selectAlternateInterface(0, 0)),
    assertRejectsWithInterfaceStateChangeInProgressError(
        device.controlTransferOut({
          requestType: 'standard',
          recipient: 'interface',
          request: 0x42,
          value: 0x1234,
          index: 0x0000,
        })),
    assertRejectsWithInterfaceStateChangeInProgressError(
        device.controlTransferOut({
          requestType: 'standard',
          recipient: 'interface',
          request: 0x42,
          value: 0x1234,
          index: 0x0000,
        }, new Uint8Array([1, 2, 3]))),
    assertRejectsWithInterfaceStateChangeInProgressError(
        device.controlTransferIn({
          requestType: 'standard',
          recipient: 'interface',
          request: 0x42,
          value: 0x1234,
          index: 0x0000
        }, 0)),
    assertRejectsWithInterfaceStateChangeInProgressError(device.close()),
  ]);
}, 'device operations reject if an interface state change is in progress');

usb_test(async () => {
  let { device } = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(1);
  await device.claimInterface(0);
  assert_true(device.configuration.interfaces[0].claimed);
  await device.close(0);
  assert_false(device.configuration.interfaces[0].claimed);
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

usb_test(async () => {
  let { device } = await getFakeDevice();
  let usbRequestTypes = ['standard', 'class', 'vendor'];
  let usbRecipients = ['device', 'interface', 'endpoint', 'other'];
  await device.open();
  await device.selectConfiguration(1);
  await device.claimInterface(0);
  await device.selectAlternateInterface(0, 0);
  for (const requestType of usbRequestTypes) {
    for (const recipient of usbRecipients) {
      let index = recipient === 'interface' ? 0x5600 : 0x5681;
      let result = await device.controlTransferIn({
        requestType: requestType,
        recipient: recipient,
        request: 0x42,
        value: 0x1234,
        index: index
      }, 7);
      assert_true(result instanceof USBInTransferResult);
      assert_equals(result.status, 'ok');
      assert_equals(result.data.byteLength, 7);
      assert_equals(result.data.getUint16(0), 0x07);
      assert_equals(result.data.getUint8(2), 0x42);
      assert_equals(result.data.getUint16(3), 0x1234);
      assert_equals(result.data.getUint16(5), index);
    }
  }
  await device.close();
}, 'can issue all types of IN control transfers');

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

usb_test(async () => {
  let { device } = await getFakeDevice();
  let usbRequestTypes = ['standard', 'class', 'vendor'];
  let usbRecipients = ['device', 'interface', 'endpoint', 'other'];
  let dataArray = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
  let dataTypes = [dataArray, dataArray.buffer];
  await device.open();
  await device.selectConfiguration(1);
  await device.claimInterface(0);
  await device.selectAlternateInterface(0, 0);
  for (const requestType of usbRequestTypes) {
    for (const recipient of usbRecipients) {
      let index = recipient === 'interface' ? 0x5600 : 0x5681;
      let transferParams = {
        requestType: requestType,
        recipient: recipient,
        request: 0x42,
        value: 0x1234,
        index: index
      };
      for (const data of dataTypes) {
        let result = await device.controlTransferOut(transferParams, data);
        assert_true(result instanceof USBOutTransferResult);
        assert_equals(result.status, 'ok');
        assert_equals(result.bytesWritten, 8);
      }
      let result = await device.controlTransferOut(transferParams);
      assert_true(result instanceof USBOutTransferResult);
      assert_equals(result.status, 'ok');
    }
  }
  await device.close();
}, 'can issue all types of OUT control transfers');

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

usb_test(async () => {
  let { device } = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(1);
  await device.claimInterface(0);
  assertRejectsWithTypeError(device.controlTransferOut({
    requestType: 'invalid',
    recipient: 'device',
    request: 0x42,
    value: 0x1234,
    index: 0x5678
  }, new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8])));
  assertRejectsWithTypeError(device.controlTransferIn({
    requestType: 'invalid',
    recipient: 'device',
    request: 0x42,
    value: 0x1234,
    index: 0x5678
  }, 0));
  await device.close();
}, 'control transfers with a invalid request type reject');

usb_test(async () => {
  let { device } = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(1);
  await device.claimInterface(0);
  assertRejectsWithTypeError(device.controlTransferOut({
    requestType: 'vendor',
    recipient: 'invalid',
    request: 0x42,
    value: 0x1234,
    index: 0x5678
  }, new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8])));
  assertRejectsWithTypeError(device.controlTransferIn({
    requestType: 'vendor',
    recipient: 'invalid',
    request: 0x42,
    value: 0x1234,
    index: 0x5678
  }, 0));
}, 'control transfers with a invalid recipient type reject');

usb_test(async () => {
  let { device } = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(1);
  await device.claimInterface(0);
  assertRejectsWithNotFoundError(device.controlTransferOut({
    requestType: 'vendor',
    recipient: 'interface',
    request: 0x42,
    value: 0x1234,
    index: 0x0002  // Last byte of index is interface number.
  }, new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8])));
  assertRejectsWithNotFoundError(device.controlTransferIn({
    requestType: 'vendor',
    recipient: 'interface',
    request: 0x42,
    value: 0x1234,
    index: 0x0002  // Last byte of index is interface number.
  }, 0));
}, 'control transfers to a non-existant interface reject');

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
