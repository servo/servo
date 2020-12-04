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
