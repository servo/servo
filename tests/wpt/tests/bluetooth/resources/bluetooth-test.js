'use strict';

/**
 * Test Setup Helpers
 */

/**
 * Loads a script by creating a <script> element pointing to |path|.
 * @param {string} path The path of the script to load.
 * @returns {Promise<void>} Resolves when the script has finished loading.
 */
function loadScript(path) {
  let script = document.createElement('script');
  let promise = new Promise(resolve => script.onload = resolve);
  script.src = path;
  script.async = false;
  document.head.appendChild(script);
  return promise;
}

/**
 * Performs the Chromium specific setup necessary to run the tests in the
 * Chromium browser. This test file is shared between Web Platform Tests and
 * Blink Web Tests, so this method figures out the correct paths to use for
 * loading scripts.
 *
 * TODO(https://crbug.com/569709): Update this description when all Web
 * Bluetooth Blink Web Tests have been migrated into this repository.
 * @returns {Promise<void>} Resolves when Chromium specific setup is complete.
 */
async function performChromiumSetup() {
  // Determine path prefixes.
  let resPrefix = '/resources';
  const chromiumResources = ['/resources/chromium/web-bluetooth-test.js'];
  const pathname = window.location.pathname;
  if (pathname.includes('/wpt_internal/')) {
    chromiumResources.push(
        '/wpt_internal/bluetooth/resources/bluetooth-fake-adapter.js');
  }

  await loadScript(`${resPrefix}/test-only-api.js`);
  if (!isChromiumBased) {
    return;
  }

  for (const path of chromiumResources) {
    await loadScript(path);
  }

  await initializeChromiumResources();

  // Call setBluetoothFakeAdapter() to clean up any fake adapters left over by
  // legacy tests. Legacy tests that use setBluetoothFakeAdapter() sometimes
  // fail to clean their fake adapter. This is not a problem for these tests
  // because the next setBluetoothFakeAdapter() will clean it up anyway but it
  // is a problem for the new tests that do not use setBluetoothFakeAdapter().
  // TODO(https://crbug.com/569709): Remove once setBluetoothFakeAdapter is no
  // longer used.
  if (typeof setBluetoothFakeAdapter !== 'undefined') {
    setBluetoothFakeAdapter('');
  }
}

/**
 * These tests rely on the User Agent providing an implementation of the Web
 * Bluetooth Testing API.
 * https://docs.google.com/document/d/1Nhv_oVDCodd1pEH_jj9k8gF4rPGb_84VYaZ9IG8M_WY/edit?ts=59b6d823#heading=h.7nki9mck5t64
 * @param {function{*}: Promise<*>} test_function The Web Bluetooth test to run.
 * @param {string} name The name or description of the test.
 * @param {object} properties An object containing extra options for the test.
 * @param {Boolean} validate_response_consumed Whether to validate all response
 *     consumed or not.
 * @returns {Promise<void>} Resolves if Web Bluetooth test ran successfully, or
 *     rejects if the test failed.
 */
function bluetooth_test(
    test_function, name, properties, validate_response_consumed = true) {
  return promise_test(async (t) => {
    assert_implements(navigator.bluetooth, 'missing navigator.bluetooth');
    // Trigger Chromium-specific setup.
    await performChromiumSetup();
    assert_implements(
        navigator.bluetooth.test, 'missing navigator.bluetooth.test');
    await test_function(t);
    if (validate_response_consumed) {
      let consumed = await navigator.bluetooth.test.allResponsesConsumed();
      assert_true(consumed);
    }
  }, name, properties);
}

/**
 * These tests rely on the User Agent providing an implementation of the
 * WebDriver-Bidi for testing Web Bluetooth
 * https://webbluetoothcg.github.io/web-bluetooth/#automated-testing
 * @param {function{*}: Promise<*>} test_function The Web Bluetooth test to run.
 * @param {string} name The name or description of the test.
 * @param {object} properties An object containing extra options for the test.
 * @param {Boolean} validate_response_consumed Whether to validate all response
 *     consumed or not.
 * @returns {Promise<void>} Resolves if Web Bluetooth test ran successfully, or
 *     rejects if the test failed.
 */
function bluetooth_bidi_test(
  test_function, name, properties, validate_response_consumed = true) {
return promise_test(async (t) => {
  assert_implements(navigator.bluetooth, 'missing navigator.bluetooth');
  await test_function(t);
}, name, properties);
}

/**
 * Test Helpers
 */

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
 * Calls requestDevice() in a context that's 'allowed to show a popup'.
 * @returns {Promise<BluetoothDevice>} Resolves with a Bluetooth device if
 *     successful or rejects with an error.
 */
function requestDeviceWithTrustedClick() {
  let args = arguments;
  return callWithTrustedClick(
      () => navigator.bluetooth.requestDevice.apply(navigator.bluetooth, args));
}

/**
 * Calls requestLEScan() in a context that's 'allowed to show a popup'.
 * @returns {Promise<BluetoothLEScan>} Resolves with the properties of the scan
 *     if successful or rejects with an error.
 */
function requestLEScanWithTrustedClick() {
  let args = arguments;
  return callWithTrustedClick(
      () => navigator.bluetooth.requestLEScan.apply(navigator.bluetooth, args));
}

/**
 * Function to test that a promise rejects with the expected error type and
 * message.
 * @param {Promise} promise
 * @param {object} expected
 * @param {string} description
 * @returns {Promise<void>} Resolves if |promise| rejected with |expected|
 *     error.
 */
function assert_promise_rejects_with_message(promise, expected, description) {
  return promise.then(
      () => {
        assert_unreached('Promise should have rejected: ' + description);
      },
      error => {
        assert_equals(error.name, expected.name, 'Unexpected Error Name:');
        if (expected.message) {
          assert_true(
              error.message.includes(expected.message),
              'Unexpected Error Message:');
        }
      });
}

/**
 * Helper class that can be created to check that an event has fired.
 */
class EventCatcher {
  /**
   * @param {EventTarget} object The object to listen for events on.
   * @param {string} event The type of event to listen for.
   */
  constructor(object, event) {
    /** @type {boolean} */
    this.eventFired = false;

    /** @type {function()} */
    let event_listener = () => {
      object.removeEventListener(event, event_listener);
      this.eventFired = true;
    };
    object.addEventListener(event, event_listener);
  }
}

/**
 * Notifies when the event |type| has fired.
 * @param {EventTarget} target The object to listen for the event.
 * @param {string} type The type of event to listen for.
 * @param {object} options Characteristics about the event listener.
 * @returns {Promise<Event>} Resolves when an event of |type| has fired.
 */
function eventPromise(target, type, options) {
  return new Promise(resolve => {
    let wrapper = function(event) {
      target.removeEventListener(type, wrapper);
      resolve(event);
    };
    target.addEventListener(type, wrapper, options);
  });
}

/**
 * The action that should occur first in assert_promise_event_order_().
 * @enum {string}
 */
const ShouldBeFirst = {
  EVENT: 'event',
  PROMISE_RESOLUTION: 'promiseresolved',
};

/**
 * Helper function to assert that events are fired and a promise resolved
 * in the correct order.
 * 'event' should be passed as |should_be_first| to indicate that the events
 * should be fired first, otherwise 'promiseresolved' should be passed.
 * Attaches |num_listeners| |event| listeners to |object|. If all events have
 * been fired and the promise resolved in the correct order, returns a promise
 * that fulfills with the result of |object|.|func()| and |event.target.value|
 * of each of event listeners. Otherwise throws an error.
 * @param {ShouldBeFirst} should_be_first Indicates whether |func| should
 *     resolve before |event| is fired.
 * @param {EventTarget} object The target object to add event listeners to.
 * @param {function(*): Promise<*>} func The function to test the resolution
 *     order for.
 * @param {string} event The event type to listen for.
 * @param {number} num_listeners The number of events to listen for.
 * @returns {Promise<*>} The return value of |func|.
 */
function assert_promise_event_order_(
    should_be_first, object, func, event, num_listeners) {
  let order = [];
  let event_promises = [];
  for (let i = 0; i < num_listeners; i++) {
    event_promises.push(new Promise(resolve => {
      let event_listener = (e) => {
        object.removeEventListener(event, event_listener);
        order.push(ShouldBeFirst.EVENT);
        resolve(e.target.value);
      };
      object.addEventListener(event, event_listener);
    }));
  }

  let func_promise = object[func]().then(result => {
    order.push(ShouldBeFirst.PROMISE_RESOLUTION);
    return result;
  });

  return Promise.all([func_promise, ...event_promises]).then((result) => {
    if (should_be_first !== order[0]) {
      throw should_be_first === ShouldBeFirst.PROMISE_RESOLUTION ?
          `'${event}' was fired before promise resolved.` :
          `Promise resolved before '${event}' was fired.`;
    }

    if (order[0] !== ShouldBeFirst.PROMISE_RESOLUTION &&
        order[order.length - 1] !== ShouldBeFirst.PROMISE_RESOLUTION) {
      throw 'Promise resolved in between event listeners.';
    }

    return result;
  });
}

/**
 * Asserts that the promise returned by |func| resolves before events of type
 * |event| are fired |num_listeners| times on |object|. See
 * assert_promise_event_order_ above for more details.
 * @param {EventTarget} object  The target object to add event listeners to.
 * @param {function(*): Promise<*>} func The function whose promise should
 *     resolve first.
 * @param {string} event The event type to listen for.
 * @param {number} num_listeners The number of events to listen for.
 * @returns {Promise<*>} The return value of |func|.
 */
function assert_promise_resolves_before_event(
    object, func, event, num_listeners = 1) {
  return assert_promise_event_order_(
      ShouldBeFirst.PROMISE_RESOLUTION, object, func, event, num_listeners);
}

/**
 * Asserts that the promise returned by |func| resolves after events of type
 * |event| are fired |num_listeners| times on |object|. See
 * assert_promise_event_order_ above for more details.
 * @param {EventTarget} object  The target object to add event listeners to.
 * @param {function(*): Promise<*>} func The function whose promise should
 *     resolve first.
 * @param {string} event The event type to listen for.
 * @param {number} num_listeners The number of events to listen for.
 * @returns {Promise<*>} The return value of |func|.
 */
function assert_promise_resolves_after_event(
    object, func, event, num_listeners = 1) {
  return assert_promise_event_order_(
      ShouldBeFirst.EVENT, object, func, event, num_listeners);
}

/**
 * Returns a promise that resolves after 100ms unless the the event is fired on
 * the object in which case the promise rejects.
 * @param {EventTarget} object The target object to listen for events.
 * @param {string} event_name The event type to listen for.
 * @returns {Promise<void>} Resolves if no events were fired.
 */
function assert_no_events(object, event_name) {
  return new Promise((resolve) => {
    let event_listener = (e) => {
      object.removeEventListener(event_name, event_listener);
      assert_unreached('Object should not fire an event.');
    };
    object.addEventListener(event_name, event_listener);
    // TODO: Remove timeout.
    // http://crbug.com/543884
    step_timeout(() => {
      object.removeEventListener(event_name, event_listener);
      resolve();
    }, 100);
  });
}

/**
 * Asserts that |properties| contains the same properties in
 * |expected_properties| with equivalent values.
 * @param {object} properties Actual object to compare.
 * @param {object} expected_properties Expected object to compare with.
 */
function assert_properties_equal(properties, expected_properties) {
  for (let key in expected_properties) {
    assert_equals(properties[key], expected_properties[key]);
  }
}

/**
 * Asserts that |data_map| contains |expected_key|, and that the uint8 values
 * for |expected_key| matches |expected_value|.
 */
function assert_data_maps_equal(data_map, expected_key, expected_value) {
  assert_true(data_map.has(expected_key));

  const value = new Uint8Array(data_map.get(expected_key).buffer);
  assert_equals(value.length, expected_value.length);
  for (let i = 0; i < value.length; ++i) {
    assert_equals(value[i], expected_value[i]);
  }
}
