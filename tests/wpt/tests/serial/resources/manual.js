let manualTestPort = null;

navigator.serial.addEventListener('disconnect', (e) => {
  if (e.target === manualTestPort) {
    manualTestPort = null;
  }
})

async function getPortForManualTest() {
  if (manualTestPort) {
    return manualTestPort;
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

  manualTestPort = await navigator.serial.requestPort({filters: []});
  assert_true(manualTestPort instanceof SerialPort);

  return manualTestPort;
}

function manual_serial_test(func, name, properties) {
  promise_test(async (test) => {
    await func(test, await getPortForManualTest());
  }, name, properties);
}
