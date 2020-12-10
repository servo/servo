'use strict';

// These tests rely on the User Agent providing an implementation of the
// FakeSerialService interface which replaces the platform-specific
// implementation of the Web Serial API with one that can be automated from
// Javascript for testing purposes.
//
// In Chromium-based browsers this implementation is provided by a polyfill
// in order to reduce the amount of test-only code shipped to users. To enable
// these tests the browser must be run with these options:
//
//   --enable-blink-features=MojoJS,MojoJSTest

async function loadChromiumResources() {
  const chromiumResources = [
    '/gen/mojo/public/mojom/base/unguessable_token.mojom.js',
    '/gen/services/device/public/mojom/serial.mojom.js',
    '/gen/third_party/blink/public/mojom/serial/serial.mojom.js',
  ];
  await loadMojoResources(chromiumResources);
}

// Returns a SerialPort instance and associated FakeSerialPort instance.
async function getFakeSerialPort(fake) {
  let token = fake.addPort();
  let fakePort = fake.getFakePort(token);

  let ports = await navigator.serial.getPorts();
  assert_equals(ports.length, 1);

  let port = ports[0];
  assert_true(port instanceof SerialPort);

  return { port, fakePort };
}

let fakeSerialService = undefined;

function serial_test(func, name, properties) {
  promise_test(async (test) => {
    assert_implements(navigator.serial, 'missing navigator.serial');
    if (fakeSerialService === undefined) {
      // Try loading a polyfill for the fake serial service.
      if (isChromiumBased) {
        await loadChromiumResources();
        await loadScript('/resources/chromium/fake-serial.js');
      }
    }
    assert_implements(fakeSerialService, 'missing fakeSerialService after initialization');

    fakeSerialService.start();
    try {
      await func(test, fakeSerialService);
    } finally {
      fakeSerialService.stop();
      fakeSerialService.reset();
    }
  }, name, properties);
}

function trustedClick() {
  return new Promise(resolve => {
    let button = document.createElement('button');
    button.textContent = 'click to continue test';
    button.style.display = 'block';
    button.style.fontSize = '20px';
    button.style.padding = '10px';
    button.onclick = () => {
      document.body.removeChild(button);
      resolve();
    };
    document.body.appendChild(button);
    test_driver.click(button);
  });
}
