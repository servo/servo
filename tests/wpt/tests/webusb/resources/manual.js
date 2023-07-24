let manualTestDevice = null;

navigator.usb.addEventListener('disconnect', (e) => {
  if (e.device === manualTestDevice) {
    manualTestDevice = null;
  }
})

async function getDeviceForManualTest() {
  if (manualTestDevice) {
    return manualTestDevice;
  }

  const button = document.createElement('button');
  button.textContent = 'Click to select a device';
  button.style.display = 'block';
  button.style.fontSize = '20px';
  button.style.padding = '10px';

  await new Promise((resolve) => {
    button.onclick = () => {
      document.body.removeChild(button);
      resolve();
    };
    document.body.appendChild(button);
  });

  manualTestDevice = await navigator.usb.requestDevice({filters: []});
  assert_true(manualTestDevice instanceof USBDevice);

  return manualTestDevice;
}

function manual_usb_test(func, name, properties) {
  promise_test(async (test) => {
    await func(test, await getDeviceForManualTest());
  }, name, properties);
}

function manual_usb_serial_test(func, name, properties) {
  promise_test(async (test) => {
    const device = await getDeviceForManualTest();
    await device.open();
    test.add_cleanup(async () => {
      if (device.opened) {
        await device.close();
      }
    });

    await device.selectConfiguration(1);

    let controlInterface = undefined;
    for (const iface of device.configuration.interfaces) {
      const alternate = iface.alternates[0];
      if (alternate.interfaceClass == 2 &&
          alternate.interfaceSubclass == 2 &&
          alternate.interfaceProtocol == 0) {
        controlInterface = iface;
        break;
      }
    }
    assert_not_equals(controlInterface, undefined,
                      'No control interface found.');

    let dataInterface = undefined;
    for (const iface of device.configuration.interfaces) {
      const alternate = iface.alternates[0];
      if (alternate.interfaceClass == 10 &&
          alternate.interfaceSubclass == 0 &&
          alternate.interfaceProtocol == 0) {
        dataInterface = iface;
        break;
      }
    }
    assert_not_equals(dataInterface, undefined, 'No data interface found.');

    await device.claimInterface(controlInterface.interfaceNumber);
    await device.claimInterface(dataInterface.interfaceNumber);

    let inEndpoint = undefined;
    for (const endpoint of dataInterface.alternate.endpoints) {
      if (endpoint.type == 'bulk' && endpoint.direction == 'in') {
        inEndpoint = endpoint;
        break;
      }
    }
    assert_not_equals(inEndpoint, undefined, 'No IN endpoint found.');

    let outEndpoint = undefined;
    for (const endpoint of dataInterface.alternate.endpoints) {
      if (endpoint.type == 'bulk' && endpoint.direction == 'out') {
        outEndpoint = endpoint;
        break;
      }
    }
    assert_not_equals(outEndpoint, undefined, 'No OUT endpoint found.');

    // Execute a SET_CONTROL_LINE_STATE command to let the device know the
    // host is ready to transmit and receive data.
    await device.controlTransferOut({
      requestType: 'class',
      recipient: 'interface',
      request: 0x22,
      value: 0x01,
      index: controlInterface.interfaceNumber,
    });

    await func(test, device, inEndpoint, outEndpoint);
  }, name, properties);
}