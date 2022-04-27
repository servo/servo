'use strict';

let fakeHidService = undefined;

function hid_test(func, name, properties) {
  promise_test(async (test) => {
    assert_implements(navigator.hid, 'missing navigator.hid');
    if (fakeHidService === undefined) {
      // Try loading a polyfill for the fake hid service.
      if (isChromiumBased) {
        const fakes = await import('/resources/chromium/fake-hid.js');
        fakeHidService = fakes.fakeHidService;
      }
    }
    assert_implements(
        fakeHidService, 'missing fakeHidService after initialization');

    fakeHidService.start();
    try {
      await func(test, fakeHidService);
    } finally {
      fakeHidService.stop();
      fakeHidService.reset();
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
