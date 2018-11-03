'use strict';

// These tests rely on the User Agent providing an implementation of the
// WebUSB Testing API (https://wicg.github.io/webusb/test/).
//
// In Chromium-based browsers this implementation is provided by a polyfill
// in order to reduce the amount of test-only code shipped to users. To enable
// these tests the browser must be run with these options:
//
//   --enable-blink-features=MojoJS,MojoJSTest

(() => {
  // Load scripts needed by the test API on context creation.
  if ('MojoInterfaceInterceptor' in self) {
    let prefix = '/resources/chromium';
    if ('window' in self) {
      if (window.location.pathname.includes('/LayoutTests/')) {
        let root = window.location.pathname.match(/.*LayoutTests/);
        prefix = `${root}/external/wpt/resources/chromium`;
      }
    }
    let scriptPath = `${prefix}/webusb-child-test.js`;
    if (typeof document == 'undefined') {
      importScripts(scriptPath);
    } else {
      let script = document.createElement('script');
      script.src = scriptPath;
      script.async = false;
      document.head.appendChild(script);
    }
  }
})();

let loadChromiumResources = Promise.resolve().then(() => {
  if (!('MojoInterfaceInterceptor' in self)) {
    // Do nothing on non-Chromium-based browsers or when the Mojo bindings are
    // not present in the global namespace.
    return;
  }

  let chain = Promise.resolve();
  [
    '/resources/chromium/mojo_bindings.js',
    '/resources/chromium/string16.mojom.js',
    '/resources/chromium/url.mojom.js',
    '/resources/chromium/device.mojom.js',
    '/resources/chromium/device_manager.mojom.js',
    '/resources/chromium/web_usb_service.mojom.js',
    '/resources/chromium/webusb-test.js',
  ].forEach(path => {
    // Use importScripts for workers.
    if (typeof document === 'undefined') {
      chain = chain.then(() => importScripts(path));
      return;
    }
    let script = document.createElement('script');
    script.src = path;
    script.async = false;
    chain = chain.then(() => new Promise(resolve => {
      script.onload = () => resolve();
    }));
    document.head.appendChild(script);
  });

  return chain;
});

function usb_test(func, name, properties) {
  promise_test(async () => {
    if (navigator.usb.test === undefined) {
      // Try loading a polyfill for the WebUSB Testing API.
      await loadChromiumResources;
    }

    await navigator.usb.test.initialize();
    try {
      await func();
    } finally {
      await navigator.usb.test.reset();
    }
  }, name, properties);
}

// Returns a promise that is resolved when the next USBConnectionEvent of the
// given type is received.
function connectionEventPromise(eventType) {
  return new Promise(resolve => {
    let eventHandler = e => {
      assert_true(e instanceof USBConnectionEvent);
      navigator.usb.removeEventListener(eventType, eventHandler);
      resolve(e.device);
    };
    navigator.usb.addEventListener(eventType, eventHandler);
  });
}

// Creates a fake device and returns a promise that resolves once the
// 'connect' event is fired for the fake device. The promise is resolved with
// an object containing the fake USB device and the corresponding USBDevice.
function getFakeDevice() {
  let promise = connectionEventPromise('connect');
  let fakeDevice = navigator.usb.test.addFakeDevice(fakeDeviceInit);
  return promise.then(device => {
    return { device: device, fakeDevice: fakeDevice };
  });
}

// Disconnects the given device and returns a promise that is resolved when it
// is done.
function waitForDisconnect(fakeDevice) {
  let promise = connectionEventPromise('disconnect');
  fakeDevice.disconnect();
  return promise;
}

function assertRejectsWithError(promise, name, message) {
  return promise.then(() => {
    assert_unreached('expected promise to reject with ' + name);
  }, error => {
    assert_equals(error.name, name);
    if (message !== undefined)
      assert_equals(error.message, message);
  });
}

function assertDeviceInfoEquals(usbDevice, deviceInit) {
  for (var property in deviceInit) {
    if (property == 'activeConfigurationValue') {
      if (deviceInit.activeConfigurationValue == 0) {
        assert_equals(usbDevice.configuration, null);
      } else {
        assert_equals(usbDevice.configuration.configurationValue,
                      deviceInit.activeConfigurationValue);
      }
    } else if (Array.isArray(deviceInit[property])) {
      assert_equals(usbDevice[property].length, deviceInit[property].length);
      for (var i = 0; i < usbDevice[property].length; ++i)
        assertDeviceInfoEquals(usbDevice[property][i], deviceInit[property][i]);
    } else {
      assert_equals(usbDevice[property], deviceInit[property], property);
    }
  }
}

function callWithTrustedClick(callback) {
  return new Promise(resolve => {
    let button = document.createElement('button');
    button.textContent = 'click to continue test';
    button.style.display = 'block';
    button.style.fontSize = '20px';
    button.style.padding = '10px';
    button.onclick = () => {
      resolve(callback());
      document.body.removeChild(button);
    };
    document.body.appendChild(button);
    test_driver.click(button);
  });
}
