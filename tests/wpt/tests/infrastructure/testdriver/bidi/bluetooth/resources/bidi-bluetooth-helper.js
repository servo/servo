'use strict';

const DEVICE_NAME = 'LE Device';
const DEVICE_ADDRESS = '09:09:09:09:09:09';
const HEART_RATE_SERVICE_UUID = '0000180d-0000-1000-8000-00805f9b34fb'
const DATE_TIME_CHARACTERISTIC_UUID = '00002a08-0000-1000-8000-00805f9b34fb'
const CHARACTERISTIC_USER_DESCRIPTION_DESCRIPTOR_UUID = '00002901-0000-1000-8000-00805f9b34fb'

/**
 * Waits until the document has finished loading.
 * @returns {Promise<void>} Resolves if the document is already completely
 *     loaded or when the 'onload' event is fired.
 */
function waitForDocumentReady() {
  return new Promise(resolve => {
    if (document.readyState === 'complete') {
      resolve();
    }

    window.addEventListener('load', () => {
      resolve();
    }, {once: true});
  });
}

/**
 * Simulates a user activation prior to running |callback|.
 * @param {Function} callback The function to run after the user activation.
 * @returns {Promise<*>} Resolves when the user activation has been simulated
 *     with the result of |callback|.
 */
async function callWithTrustedClick(callback) {
  await waitForDocumentReady();
  return new Promise(resolve => {
    let button = document.createElement('button');
    button.textContent = 'click to continue test';
    button.style.display = 'block';
    button.style.fontSize = '20px';
    button.style.padding = '10px';
    button.onclick = () => {
      document.body.removeChild(button);
      resolve(callback());
    };
    document.body.appendChild(button);
    test_driver.click(button);
  });
}

/**
 * Register a one-time handler that selects the first device in the device
 * prompt upon a device prompt updated event.
 * @returns {Promise} fulfilled after the bluetooth device prompt
 * is handled, or rejected if the operation fails.
 */
function selectFirstDeviceOnDevicePromptUpdated() {
  return test_driver.bidi.bluetooth.request_device_prompt_updated.once().then(
      (promptEvent) => {
        assert_greater_than(promptEvent.devices.length, 0);
        return test_driver.bidi.bluetooth.handle_request_device_prompt({
          prompt: promptEvent.prompt,
          accept: true,
          device: promptEvent.devices[0].id
        });
      });
}

/**
 * Create a GATT connection to the `device` by registering a one-time handler
 * that simulate a successful GATT connection response upon a GATT connection
 * attempted event and making a GATT connection to it.
 * @returns {Promise} fulfilled after the GATT connection is created, or
 * rejected if the operation fails.
 */
async function createGattConnection(device) {
  const simulationProcessedPromise =
      test_driver.bidi.bluetooth.gatt_connection_attempted.once().then(
          (event) => {
            return test_driver.bidi.bluetooth.simulate_gatt_connection_response({
                address: event.address,
                code: 0x0,
            });
          });
  const connectPromise = device.gatt.connect();
  await Promise.all([connectPromise, simulationProcessedPromise]);
}

/**
 * Calls requestDevice() in a context that's 'allowed to show a popup'.
 * @returns {Promise<BluetoothDevice>} Resolves with a Bluetooth device if
 *     successful or rejects with an error.
 */
function requestDeviceWithTrustedClick(...args) {
  return callWithTrustedClick(
      () => navigator.bluetooth.requestDevice(...args));
}

/**
 * This is a test helper to run promise_test with Bluetooth emulation setup
 * before the test and teardown after the test.
 * @param {function{*}: Promise<*>} test_function The test to run.
 * @param {string} name The name or description of the test.
 * @returns {Promise<void>} Resolves if the test ran successfully, or rejects if
 * the test failed.
 */
function bluetooth_test(test_function, name) {
  return promise_test(async (t) => {
    assert_implements(navigator.bluetooth, 'missing navigator.bluetooth');
    await test_driver.bidi.bluetooth.simulate_adapter({
        state: "powered-on"
    });
    await test_driver.bidi.bluetooth.simulate_preconnected_peripheral({
        address: DEVICE_ADDRESS,
        name: DEVICE_NAME,
        manufacturerData: [],
        knownServiceUuids: []
    });
    try {
      await test_function(t);
    } finally {
      await test_driver.bidi.bluetooth.disable_simulation();
    }
  }, name);
}
