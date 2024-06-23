// META: timeout=long
// META: script=/resources/test-only-api.js
// META: script=/webusb/resources/fake-devices.js
// META: script=/webusb/resources/usb-helpers.js
'use strict';

function detachBuffer(buffer) {
  if (self.GLOBAL.isWindow())
    window.postMessage('', '*', [buffer]);
  else
    self.postMessage('', [buffer]);
}

usb_test((t) => {
  return getFakeDevice().then(({device, fakeDevice}) => {
    return waitForDisconnect(fakeDevice)
        .then(() => promise_rejects_dom(t, 'NotFoundError', device.open()));
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

usb_test(async (t) => {
  let { device } = await getFakeDevice();
  await Promise.all([
    device.open(),
    promise_rejects_dom(t, 'InvalidStateError', device.open()),
    promise_rejects_dom(t, 'InvalidStateError', device.close()),
  ]);
  await Promise.all([
    device.close(),
    promise_rejects_dom(t, 'InvalidStateError', device.open()),
    promise_rejects_dom(t, 'InvalidStateError', device.close()),
  ]);
}, 'open and close cannot be called again while open or close are in progress');

usb_test(async (t) => {
  let { device } = await getFakeDevice();
  await device.open();
  return Promise.all([
    device.selectConfiguration(1),
    promise_rejects_dom(t, 'InvalidStateError', device.claimInterface(0)),
    promise_rejects_dom(t, 'InvalidStateError', device.releaseInterface(0)),
    promise_rejects_dom(t, 'InvalidStateError', device.open()),
    promise_rejects_dom(t, 'InvalidStateError', device.selectConfiguration(1)),
    promise_rejects_dom(t, 'InvalidStateError', device.reset()),
    promise_rejects_dom(
        t, 'InvalidStateError', device.selectAlternateInterface(0, 0)),
    promise_rejects_dom(t, 'InvalidStateError', device.controlTransferOut({
      requestType: 'standard',
      recipient: 'interface',
      request: 0x42,
      value: 0x1234,
      index: 0x0000,
    })),
    promise_rejects_dom(
        t, 'InvalidStateError',
        device.controlTransferOut(
            {
              requestType: 'standard',
              recipient: 'interface',
              request: 0x42,
              value: 0x1234,
              index: 0x0000,
            },
            new Uint8Array([1, 2, 3]))),
    promise_rejects_dom(
        t, 'InvalidStateError',
        device.controlTransferIn(
            {
              requestType: 'standard',
              recipient: 'interface',
              request: 0x42,
              value: 0x1234,
              index: 0x0000
            },
            0)),
    promise_rejects_dom(t, 'InvalidStateError', device.close()),
  ]);
}, 'device operations reject if an device state change is in progress');

usb_test((t) => {
  return getFakeDevice().then(({device, fakeDevice}) => {
    return device.open()
        .then(() => waitForDisconnect(fakeDevice))
        .then(() => promise_rejects_dom(t, 'NotFoundError', device.close()));
  });
}, 'close rejects when called on a disconnected device');

usb_test((t) => {
  return getFakeDevice().then(({device, fakeDevice}) => {
    return device.open()
        .then(() => waitForDisconnect(fakeDevice))
        .then(
            () => promise_rejects_dom(
                t, 'NotFoundError', device.selectConfiguration(1)));
  });
}, 'selectConfiguration rejects when called on a disconnected device');

usb_test((t) => {
  return getFakeDevice().then(({device}) => Promise.all([
    promise_rejects_dom(t, 'InvalidStateError', device.selectConfiguration(1)),
    promise_rejects_dom(t, 'InvalidStateError', device.claimInterface(0)),
    promise_rejects_dom(t, 'InvalidStateError', device.releaseInterface(0)),
    promise_rejects_dom(
        t, 'InvalidStateError', device.selectAlternateInterface(0, 1)),
    promise_rejects_dom(
        t, 'InvalidStateError',
        device.controlTransferIn(
            {
              requestType: 'vendor',
              recipient: 'device',
              request: 0x42,
              value: 0x1234,
              index: 0x5678
            },
            7)),
    promise_rejects_dom(
        t, 'InvalidStateError',
        device.controlTransferOut(
            {
              requestType: 'vendor',
              recipient: 'device',
              request: 0x42,
              value: 0x1234,
              index: 0x5678
            },
            new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]))),
    promise_rejects_dom(t, 'InvalidStateError', device.clearHalt('in', 1)),
    promise_rejects_dom(t, 'InvalidStateError', device.transferIn(1, 8)),
    promise_rejects_dom(
        t, 'InvalidStateError', device.transferOut(1, new ArrayBuffer(8))),
    promise_rejects_dom(
        t, 'InvalidStateError', device.isochronousTransferIn(1, [8])),
    promise_rejects_dom(
        t, 'InvalidStateError',
        device.isochronousTransferOut(1, new ArrayBuffer(8), [8])),
    promise_rejects_dom(t, 'InvalidStateError', device.reset())
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

usb_test((t) => {
  return getFakeDevice().then(({ device }) => {
    assert_equals(device.configuration, null);
    return device.open()
        .then(
            () => promise_rejects_dom(
                t, 'NotFoundError', device.selectConfiguration(10)))
        .then(() => device.close());
  });
}, 'selectConfiguration rejects on invalid configurations');

usb_test((t) => {
  return getFakeDevice().then(({ device }) => {
    assert_equals(device.configuration, null);
    return device.open()
        .then(() => Promise.all([
          promise_rejects_dom(t, 'InvalidStateError', device.claimInterface(0)),
          promise_rejects_dom(
              t, 'InvalidStateError', device.releaseInterface(0)),
          promise_rejects_dom(
              t, 'InvalidStateError', device.selectAlternateInterface(0, 1)),
          promise_rejects_dom(
              t, 'InvalidStateError', device.clearHalt('in', 1)),
          promise_rejects_dom(t, 'InvalidStateError', device.transferIn(1, 8)),
          promise_rejects_dom(
              t, 'InvalidStateError',
              device.transferOut(1, new ArrayBuffer(8))),
          promise_rejects_dom(
              t, 'InvalidStateError', device.isochronousTransferIn(1, [8])),
          promise_rejects_dom(
              t, 'InvalidStateError',
              device.isochronousTransferOut(1, new ArrayBuffer(8), [8])),
        ]))
        .then(() => device.close());
  });
}, 'methods requiring it reject when the device is unconfigured');

usb_test(async () => {
  let { device } = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(1);
  assert_false(device.configuration.interfaces[0].claimed);
  assert_false(device.configuration.interfaces[1].claimed);

  await device.claimInterface(0);
  assert_true(device.configuration.interfaces[0].claimed);
  assert_false(device.configuration.interfaces[1].claimed);

  await device.claimInterface(1);
  assert_true(device.configuration.interfaces[0].claimed);
  assert_true(device.configuration.interfaces[1].claimed);

  await device.releaseInterface(0);
  assert_false(device.configuration.interfaces[0].claimed);
  assert_true(device.configuration.interfaces[1].claimed);

  await device.releaseInterface(1);
  assert_false(device.configuration.interfaces[0].claimed);
  assert_false(device.configuration.interfaces[1].claimed);

  await device.close();
}, 'interfaces can be claimed and released');

usb_test(async () => {
  let { device } = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(1);
  assert_false(device.configuration.interfaces[0].claimed);
  assert_false(device.configuration.interfaces[1].claimed);

  await Promise.all([device.claimInterface(0),
                     device.claimInterface(1)]);
  assert_true(device.configuration.interfaces[0].claimed);
  assert_true(device.configuration.interfaces[1].claimed);

  await Promise.all([device.releaseInterface(0),
                     device.releaseInterface(1)]);
  assert_false(device.configuration.interfaces[0].claimed);
  assert_false(device.configuration.interfaces[1].claimed);

  await device.close();
}, 'interfaces can be claimed and released in parallel');

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

usb_test(async (t) => {
  let { device } = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(1);
  return Promise.all([
    device.claimInterface(0),
    promise_rejects_dom(t, 'InvalidStateError', device.claimInterface(0)),
    promise_rejects_dom(t, 'InvalidStateError', device.releaseInterface(0)),
    promise_rejects_dom(t, 'InvalidStateError', device.open()),
    promise_rejects_dom(t, 'InvalidStateError', device.selectConfiguration(1)),
    promise_rejects_dom(t, 'InvalidStateError', device.reset()),
    promise_rejects_dom(
        t, 'InvalidStateError', device.selectAlternateInterface(0, 0)),
    promise_rejects_dom(t, 'InvalidStateError', device.controlTransferOut({
      requestType: 'standard',
      recipient: 'interface',
      request: 0x42,
      value: 0x1234,
      index: 0x0000,
    })),
    promise_rejects_dom(
        t, 'InvalidStateError',
        device.controlTransferOut(
            {
              requestType: 'standard',
              recipient: 'interface',
              request: 0x42,
              value: 0x1234,
              index: 0x0000,
            },
            new Uint8Array([1, 2, 3]))),
    promise_rejects_dom(
        t, 'InvalidStateError',
        device.controlTransferIn(
            {
              requestType: 'standard',
              recipient: 'interface',
              request: 0x42,
              value: 0x1234,
              index: 0x0000
            },
            0)),
    promise_rejects_dom(t, 'InvalidStateError', device.close()),
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

usb_test((t) => {
  return getFakeDevice().then(({device}) => {
    return device.open()
        .then(() => device.selectConfiguration(1))
        .then(() => Promise.all([
          promise_rejects_dom(t, 'NotFoundError', device.claimInterface(2)),
          promise_rejects_dom(t, 'NotFoundError', device.releaseInterface(2)),
        ]))
        .then(() => device.close());
  });
}, 'a non-existent interface cannot be claimed or released');

usb_test((t) => {
  return getFakeDevice().then(({device, fakeDevice}) => {
    return device.open()
        .then(() => device.selectConfiguration(1))
        .then(() => waitForDisconnect(fakeDevice))
        .then(
            () => promise_rejects_dom(
                t, 'NotFoundError', device.claimInterface(0)));
  });
}, 'claimInterface rejects when called on a disconnected device');

usb_test((t) => {
  return getFakeDevice().then(({device, fakeDevice}) => {
    return device.open()
        .then(() => device.selectConfiguration(1))
        .then(() => device.claimInterface(0))
        .then(() => waitForDisconnect(fakeDevice))
        .then(
            () => promise_rejects_dom(
                t, 'NotFoundError', device.releaseInterface(0)));
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

usb_test(
    async () => {
      const {device} = await getFakeDevice();
      await device.open();
      await device.selectConfiguration(3);
      await device.claimInterface(2);
      await device.selectAlternateInterface(2, 0);
      await device.close();
    },
    'can select an alternate interface on a setting with non-sequential ' +
        'interface number');

usb_test(
    async () => {
      const {device} = await getFakeDevice();
      await device.open();
      await device.selectConfiguration(3);
      await device.claimInterface(0);
      await device.selectAlternateInterface(0, 2);
      await device.close();
    },
    'can select an alternate interface on a setting with non-sequential ' +
        'alternative setting value');

usb_test((t) => {
  return getFakeDevice().then(({device}) => {
    return device.open()
        .then(() => device.selectConfiguration(2))
        .then(() => device.claimInterface(0))
        .then(
            () => promise_rejects_dom(
                t, 'NotFoundError', device.selectAlternateInterface(0, 2)))
        .then(() => device.close());
  });
}, 'cannot select a non-existent alternate interface');

usb_test((t) => {
  return getFakeDevice().then(({device, fakeDevice}) => {
    return device.open()
        .then(() => device.selectConfiguration(2))
        .then(() => device.claimInterface(0))
        .then(() => waitForDisconnect(fakeDevice))
        .then(
            () => promise_rejects_dom(
                t, 'NotFoundError', device.selectAlternateInterface(0, 1)));
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

usb_test(async () => {
  let { device } = await getFakeDevice();
  let usbRequestTypes = ['standard', 'class', 'vendor'];
  let usbRecipients = ['device', 'other'];
  await device.open();
  await Promise.all(usbRequestTypes.flatMap(requestType => {
    return usbRecipients.map(async recipient => {
      let result = await device.controlTransferIn({
        requestType: requestType,
        recipient: recipient,
        request: 0x42,
        value: 0x1234,
        index: 0x5678
      }, 7);
      assert_true(result instanceof USBInTransferResult);
      assert_equals(result.status, 'ok');
      assert_equals(result.data.byteLength, 7);
      assert_equals(result.data.getUint16(0), 0x07);
      assert_equals(result.data.getUint8(2), 0x42);
      assert_equals(result.data.getUint16(3), 0x1234);
      assert_equals(result.data.getUint16(5), 0x5678);
    });
  }));
  await device.close();
}, 'device-scope IN control transfers don\'t require configuration');

usb_test(async (t) => {
  let { device } = await getFakeDevice();
  let usbRequestTypes = ['standard', 'class', 'vendor'];
  let usbRecipients = ['interface', 'endpoint'];
  await device.open();
  await Promise.all(usbRequestTypes.flatMap(requestType => {
    return usbRecipients.map(recipient => {
      let index = recipient === 'interface' ? 0x5600 : 0x5681;
      return promise_rejects_dom(
          t, 'InvalidStateError',
          device.controlTransferIn(
              {
                requestType: requestType,
                recipient: recipient,
                request: 0x42,
                value: 0x1234,
                index: index
              },
              7));
    });
  }));
  await device.close();
}, 'interface-scope IN control transfers require configuration');

usb_test(async (t) => {
  let { device } = await getFakeDevice();
  let usbRequestTypes = ['standard', 'class', 'vendor'];
  let usbRecipients = ['interface', 'endpoint'];
  await device.open();
  await device.selectConfiguration(1);
  await Promise.all(usbRequestTypes.flatMap(requestType => {
    return [
      promise_rejects_dom(
          t, 'InvalidStateError',
          device.controlTransferIn(
              {
                requestType: requestType,
                recipient: 'interface',
                request: 0x42,
                value: 0x1234,
                index: 0x5600
              },
              7)),
      promise_rejects_dom(
          t, 'NotFoundError',
          device.controlTransferIn(
              {
                requestType: requestType,
                recipient: 'endpoint',
                request: 0x42,
                value: 0x1234,
                index: 0x5681
              },
              7))
    ];
  }));
  await device.close();
}, 'interface-scope IN control transfers require claiming the interface');

usb_test((t) => {
  return getFakeDevice().then(({device, fakeDevice}) => {
    return device.open()
        .then(() => device.selectConfiguration(1))
        .then(() => waitForDisconnect(fakeDevice))
        .then(
            () => promise_rejects_dom(
                t, 'NotFoundError',
                device.controlTransferIn(
                    {
                      requestType: 'vendor',
                      recipient: 'device',
                      request: 0x42,
                      value: 0x1234,
                      index: 0x5678
                    },
                    7)));
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

usb_test(async () => {
  let { device } = await getFakeDevice();
  let usbRequestTypes = ['standard', 'class', 'vendor'];
  let usbRecipients = ['device', 'other'];
  let dataArray = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
  let dataTypes = [dataArray, dataArray.buffer];
  await device.open();
  await Promise.all(usbRequestTypes.flatMap(requestType => {
    return usbRecipients.flatMap(recipient => {
      let transferParams = {
        requestType: requestType,
        recipient: recipient,
        request: 0x42,
        value: 0x1234,
        index: 0x5678
      };
      return dataTypes.map(async data => {
        let result = await device.controlTransferOut(transferParams, data);
        assert_true(result instanceof USBOutTransferResult);
        assert_equals(result.status, 'ok');
        assert_equals(result.bytesWritten, 8);
      }).push((async () => {
        let result = await device.controlTransferOut(transferParams);
        assert_true(result instanceof USBOutTransferResult);
        assert_equals(result.status, 'ok');
      })());
    });
  }));
  await device.close();
}, 'device-scope OUT control transfers don\'t require configuration');

usb_test(async (t) => {
  let { device } = await getFakeDevice();
  let usbRequestTypes = ['standard', 'class', 'vendor'];
  let usbRecipients = ['interface', 'endpoint'];
  let dataArray = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
  let dataTypes = [dataArray, dataArray.buffer];
  await device.open();
  await Promise.all(usbRequestTypes.flatMap(requestType => {
    return usbRecipients.flatMap(recipient => {
      let index = recipient === 'interface' ? 0x5600 : 0x5681;
      let transferParams = {
        requestType: requestType,
        recipient: recipient,
        request: 0x42,
        value: 0x1234,
        index: index
      };
      return dataTypes
          .map(data => {
            return promise_rejects_dom(
                t, 'InvalidStateError',
                device.controlTransferOut(transferParams, data));
          })
          .push(promise_rejects_dom(
              t, 'InvalidStateError',
              device.controlTransferOut(transferParams)));
    });
  }));
  await device.close();
}, 'interface-scope OUT control transfers require configuration');

usb_test(async (t) => {
  let { device } = await getFakeDevice();
  let usbRequestTypes = ['standard', 'class', 'vendor'];
  let usbRecipients = ['interface', 'endpoint'];
  let dataArray = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
  let dataTypes = [dataArray, dataArray.buffer];
  await device.open();
  await device.selectConfiguration(1);
  await Promise.all(usbRequestTypes.flatMap(requestType => {
    return usbRecipients.flatMap(recipient => {
      let index = recipient === 'interface' ? 0x5600 : 0x5681;
      let error =
          recipient === 'interface' ? 'InvalidStateError' : 'NotFoundError';
      let transferParams = {
        requestType: requestType,
        recipient: recipient,
        request: 0x42,
        value: 0x1234,
        index: index
      };
      return dataTypes
          .map(data => {
            return promise_rejects_dom(
                t, error, device.controlTransferOut(transferParams, data));
          })
          .push(promise_rejects_dom(
              t, error, device.controlTransferOut(transferParams)));
    });
  }));
  await device.close();
}, 'interface-scope OUT control transfers an interface claim');

usb_test((t) => {
  return getFakeDevice().then(({device, fakeDevice}) => {
    return device.open()
        .then(() => device.selectConfiguration(1))
        .then(() => waitForDisconnect(fakeDevice))
        .then(
            () => promise_rejects_dom(
                t, 'NotFoundError',
                device.controlTransferOut(
                    {
                      requestType: 'vendor',
                      recipient: 'device',
                      request: 0x42,
                      value: 0x1234,
                      index: 0x5678
                    },
                    new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]))));
  });
}, 'controlTransferOut rejects when called on a disconnected device');

usb_test(async (t) => {
  let { device } = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(1);
  await device.claimInterface(0);
  await Promise.all([
    promise_rejects_js(
        t, TypeError,
        device.controlTransferOut(
            {
              requestType: 'invalid',
              recipient: 'device',
              request: 0x42,
              value: 0x1234,
              index: 0x5678
            },
            new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]))),
    promise_rejects_js(
        t, TypeError,
        device.controlTransferIn(
            {
              requestType: 'invalid',
              recipient: 'device',
              request: 0x42,
              value: 0x1234,
              index: 0x5678
            },
            0)),
  ]);
  await device.close();
}, 'control transfers with a invalid request type reject');

usb_test(async (t) => {
  let { device } = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(1);
  await device.claimInterface(0);
  await Promise.all([
    promise_rejects_js(
        t, TypeError,
        device.controlTransferOut(
            {
              requestType: 'vendor',
              recipient: 'invalid',
              request: 0x42,
              value: 0x1234,
              index: 0x5678
            },
            new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]))),
    promise_rejects_js(
        t, TypeError,
        device.controlTransferIn(
            {
              requestType: 'vendor',
              recipient: 'invalid',
              request: 0x42,
              value: 0x1234,
              index: 0x5678
            },
            0)),
  ]);
}, 'control transfers with a invalid recipient type reject');

usb_test(async (t) => {
  let { device } = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(1);
  await device.claimInterface(0);
  await Promise.all([
    promise_rejects_dom(
        t, 'NotFoundError',
        device.controlTransferOut(
            {
              requestType: 'vendor',
              recipient: 'interface',
              request: 0x42,
              value: 0x1234,
              index: 0x0002  // Last byte of index is interface number.
            },
            new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]))),
    promise_rejects_dom(
        t, 'NotFoundError',
        device.controlTransferIn(
            {
              requestType: 'vendor',
              recipient: 'interface',
              request: 0x42,
              value: 0x1234,
              index: 0x0002  // Last byte of index is interface number.
            },
            0)),
  ]);
}, 'control transfers to a non-existant interface reject');

usb_test((t) => {
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
          promise_rejects_dom(
              t, 'InvalidStateError',
              device.controlTransferIn(interfaceRequest, 7)),
          promise_rejects_dom(
              t, 'NotFoundError', device.controlTransferIn(endpointRequest, 7)),
          promise_rejects_dom(
              t, 'InvalidStateError',
              device.controlTransferOut(interfaceRequest, data)),
          promise_rejects_dom(
              t, 'NotFoundError',
              device.controlTransferOut(endpointRequest, data)),
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

usb_test(async () => {
  const { device } = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(1);
  await device.claimInterface(0);

  const transfer_params = {
      requestType: 'vendor',
      recipient: 'device',
      request: 0,
      value: 0,
      index: 0
  };

  try {
    const array_buffer = new ArrayBuffer(64 * 8);
    const result =
        await device.controlTransferOut(transfer_params, array_buffer);
    assert_equals(result.status, 'ok');

    detachBuffer(array_buffer);
    await device.controlTransferOut(transfer_params, array_buffer);
    assert_unreached();
  } catch (e) {
    assert_equals(e.code, DOMException.INVALID_STATE_ERR);
  }

  try {
    const typed_array = new Uint8Array(64 * 8);
    const result =
        await device.controlTransferOut(transfer_params, typed_array);
    assert_equals(result.status, 'ok');

    detachBuffer(typed_array.buffer);
    await device.controlTransferOut(transfer_params, typed_array);
    assert_unreached();
  } catch (e) {
    assert_equals(e.code, DOMException.INVALID_STATE_ERR);
  }
}, 'controlTransferOut rejects if called with a detached buffer');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => device.claimInterface(0))
      .then(() => device.clearHalt('in', 1))
      .then(() => device.close());
  });
}, 'can clear a halt condition');

usb_test((t) => {
  return getFakeDevice(t).then(({device, fakeDevice}) => {
    return device.open()
        .then(() => device.selectConfiguration(1))
        .then(() => device.claimInterface(0))
        .then(() => waitForDisconnect(fakeDevice))
        .then(
            () => promise_rejects_dom(
                t, 'NotFoundError', device.clearHalt('in', 1)));
  });
}, 'clearHalt rejects when called on a disconnected device');

usb_test((t) => {
  return getFakeDevice().then(({ device }) => {
    let data = new DataView(new ArrayBuffer(1024));
    for (let i = 0; i < 1024; ++i)
      data.setUint8(i, i & 0xff);
    return device.open()
        .then(() => device.selectConfiguration(1))
        .then(() => device.claimInterface(0))
        .then(() => Promise.all([
          promise_rejects_dom(
              t, 'NotFoundError', device.transferIn(2, 8)),  // Unclaimed
          promise_rejects_dom(
              t, 'NotFoundError', device.transferIn(3, 8)),  // Non-existent
          promise_rejects_dom(t, 'IndexSizeError', device.transferIn(16, 8)),
          promise_rejects_dom(
              t, 'NotFoundError', device.transferOut(2, data)),  // Unclaimed
          promise_rejects_dom(
              t, 'NotFoundError', device.transferOut(3, data)),  // Non-existent
          promise_rejects_dom(
              t, 'IndexSizeError', device.transferOut(16, data)),
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

usb_test((t) => {
  return getFakeDevice().then(({device, fakeDevice}) => {
    return device.open()
        .then(() => device.selectConfiguration(1))
        .then(() => device.claimInterface(1))
        .then(() => waitForDisconnect(fakeDevice))
        .then(
            () => promise_rejects_dom(
                t, 'NotFoundError', device.transferIn(2, 1024)));
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

usb_test((t) => {
  return getFakeDevice().then(({ device, fakeDevice }) => {
    return device.open()
      .then(() => device.selectConfiguration(1))
      .then(() => device.claimInterface(1))
      .then(() => {
        let data = new DataView(new ArrayBuffer(1024));
        for (let i = 0; i < 1024; ++i)
          data.setUint8(i, i & 0xff);
        return waitForDisconnect(fakeDevice)
            .then(
                () => promise_rejects_dom(
                    t, 'NotFoundError', device.transferOut(2, data)));
      });
  });
}, 'transferOut rejects if called on a disconnected device');

usb_test(async () => {
  const { device } = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(1);
  await device.claimInterface(1);


  try {
    const array_buffer = new ArrayBuffer(64 * 8);
    const result = await device.transferOut(2, array_buffer);
    assert_equals(result.status, 'ok');

    detachBuffer(array_buffer);
    await device.transferOut(2, array_buffer);
    assert_unreached();
  } catch (e) {
    assert_equals(e.code, DOMException.INVALID_STATE_ERR);
  }

  try {
    const typed_array = new Uint8Array(64 * 8);
    const result = await device.transferOut(2, typed_array);
    assert_equals(result.status, 'ok');

    detachBuffer(typed_array.buffer);
    await device.transferOut(2, typed_array);
    assert_unreached();
  } catch (e) {
    assert_equals(e.code, DOMException.INVALID_STATE_ERR);
  }
}, 'transferOut rejects if called with a detached buffer');

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

usb_test((t) => {
  return getFakeDevice().then(({device, fakeDevice}) => {
    return device.open()
        .then(() => device.selectConfiguration(2))
        .then(() => device.claimInterface(0))
        .then(() => device.selectAlternateInterface(0, 1))
        .then(() => waitForDisconnect(fakeDevice))
        .then(
            () => promise_rejects_dom(
                t, 'NotFoundError',
                device.isochronousTransferIn(
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

usb_test((t) => {
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
            .then(
                () => promise_rejects_dom(
                    t, 'NotFoundError',
                    device.isochronousTransferOut(
                        1, data, [64, 64, 64, 64, 64, 64, 64, 64])));
      });
  });
}, 'isochronousTransferOut rejects when called on a disconnected device');

usb_test(async () => {
  const { device } = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(2);
  await device.claimInterface(0);
  await device.selectAlternateInterface(0, 1);


  try {
    const array_buffer = new ArrayBuffer(64 * 8);
    const result = await device.isochronousTransferOut(
        1, array_buffer, [64, 64, 64, 64, 64, 64, 64, 64]);
    for (let i = 0; i < result.packets.length; ++i)
      assert_equals(result.packets[i].status, 'ok');

    detachBuffer(array_buffer);
    await device.isochronousTransferOut(
        1, array_buffer, [64, 64, 64, 64, 64, 64, 64, 64]);
    assert_unreached();
  } catch (e) {
    assert_equals(e.code, DOMException.INVALID_STATE_ERR);
  }

  try {
    const typed_array = new Uint8Array(64 * 8);
    const result = await device.isochronousTransferOut(
        1, typed_array, [64, 64, 64, 64, 64, 64, 64, 64]);
    for (let i = 0; i < result.packets.length; ++i)
      assert_equals(result.packets[i].status, 'ok');

    detachBuffer(typed_array.buffer);
    await device.isochronousTransferOut(
        1, typed_array, [64, 64, 64, 64, 64, 64, 64, 64]);
    assert_unreached();
  } catch (e) {
    assert_equals(e.code, DOMException.INVALID_STATE_ERR);
  }
}, 'isochronousTransferOut rejects when called with a detached buffer');

usb_test(() => {
  return getFakeDevice().then(({ device }) => {
    return device.open().then(() => device.reset()).then(() => device.close());
  });
}, 'can reset the device');

usb_test((t) => {
  return getFakeDevice().then(({device, fakeDevice}) => {
    return device.open()
        .then(() => waitForDisconnect(fakeDevice))
        .then(() => promise_rejects_dom(t, 'NotFoundError', device.reset()));
  });
}, 'resetDevice rejects when called on a disconnected device');

usb_test(async (t) => {
  const PACKET_COUNT = 4;
  const PACKET_LENGTH = 8;
  const {device, fakeDevice} = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(2);
  await device.claimInterface(0);
  await device.selectAlternateInterface(0, 1);
  const buffer = new Uint8Array(PACKET_COUNT * PACKET_LENGTH);
  const packetLengths = new Array(PACKET_COUNT).fill(PACKET_LENGTH);
  packetLengths[0] = PACKET_LENGTH - 1;
  await promise_rejects_dom(
      t, 'DataError', device.isochronousTransferOut(1, buffer, packetLengths));
}, 'isochronousTransferOut rejects when buffer size exceeds packet lengths');

usb_test(async (t) => {
  const PACKET_COUNT = 4;
  const PACKET_LENGTH = 8;
  const {device, fakeDevice} = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(2);
  await device.claimInterface(0);
  await device.selectAlternateInterface(0, 1);
  const buffer = new Uint8Array(PACKET_COUNT * PACKET_LENGTH);
  const packetLengths = new Array(PACKET_COUNT).fill(PACKET_LENGTH);
  packetLengths[0] = PACKET_LENGTH + 1;
  await promise_rejects_dom(
      t, 'DataError', device.isochronousTransferOut(1, buffer, packetLengths));
}, 'isochronousTransferOut rejects when packet lengths exceed buffer size');

usb_test(async (t) => {
  const {device} = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(2);
  await device.claimInterface(0);
  await device.selectAlternateInterface(0, 1);
  const packetLengths = [33554432, 1];
  await promise_rejects_dom(
      t, 'DataError', device.isochronousTransferIn(1, packetLengths));
}, 'isochronousTransferIn rejects when packet lengths exceed maximum size');

usb_test(async (t) => {
  const {device} = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(2);
  await device.claimInterface(0);
  await device.selectAlternateInterface(0, 1);
  const buffer = new Uint8Array(33554432 + 1);
  const packetLengths = [33554432, 1];
  await promise_rejects_dom(
      t, 'DataError', device.isochronousTransferOut(1, buffer, packetLengths));
}, 'isochronousTransferOut rejects when packet lengths exceed maximum size');

usb_test(async (t) => {
  const {device} = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(2);
  await device.claimInterface(0);
  await device.selectAlternateInterface(0, 1);
  await promise_rejects_dom(
      t, 'DataError', device.transferIn(1, 33554433));
}, 'transferIn rejects when packet lengths exceed maximum size');

usb_test(async (t) => {
  const {device} = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(2);
  await device.claimInterface(0);
  await device.selectAlternateInterface(0, 1);
  await promise_rejects_dom(
      t, 'DataError', device.transferOut(1, new ArrayBuffer(33554433)));
}, 'transferOut rejects when packet lengths exceed maximum size');

usb_test(async (t) => {
  const {device} = await getFakeDevice();
  await device.open();
  await device.selectConfiguration(2);
  await device.claimInterface(0);
  await device.selectAlternateInterface(0, 1);
  await promise_rejects_dom(
      t, 'DataError', device.controlTransferOut({
        requestType: 'vendor',
        recipient: 'device',
        request: 0x42,
        value: 0x1234,
        index: 0x5678
      }, new ArrayBuffer(33554433)));
}, 'controlTransferOut rejects when packet lengths exceed maximum size');
