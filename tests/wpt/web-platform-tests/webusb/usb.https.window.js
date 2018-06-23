// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/webusb/resources/fake-devices.js
// META: script=/webusb/resources/usb-helpers.js
'use strict';

usb_test(() => {
  return navigator.usb.requestDevice({ filters: [] })
    .then(device => {
      assert_unreachable('requestDevice should reject without a user gesture');
    })
    .catch(error => {
      assert_equals(error.code, DOMException.SECURITY_ERR);
    });
}, 'requestDevice rejects when called without a user gesture');

usb_test(() => {
  return callWithTrustedClick(() => navigator.usb.requestDevice({ filters: [] })
    .then(device => {
      assert_unreachable('requestDevice should reject when no device selected');
    })
    .catch(error => {
      assert_equals(error.code, DOMException.NOT_FOUND_ERR);
    })
  );
}, 'requestDevice rejects when no device is chosen');

usb_test(() => {
  return getFakeDevice().then(({ device, fakeDevice }) => {
    navigator.usb.test.onrequestdevice = event => {
      navigator.usb.test.onrequestdevice = undefined;
      event.respondWith(fakeDevice);
    };
    return callWithTrustedClick(() => {
      return navigator.usb.requestDevice({ filters: [] }).then(chosenDevice => {
        assert_equals(chosenDevice, device);
      });
    });
  });
}, 'requestDevice returns the device chosen by the user');

usb_test(() => {
  return getFakeDevice().then(({ device, fakeDevice }) => {
    navigator.usb.test.onrequestdevice = event => {
      navigator.usb.test.onrequestdevice = undefined;
      event.respondWith(fakeDevice);
    };
    return callWithTrustedClick(() => {
      return navigator.usb.requestDevice({ filters: [] }).then(chosenDevice => {
        assert_equals(chosenDevice, device);
        return navigator.usb.getDevices().then(devices => {
          assert_equals(devices.length, 1);
          assert_equals(devices[0], chosenDevice);
        });
      });
    });
  });
}, 'getDevices returns the same object as requestDevice');

usb_test(() => {
  const expectedFilters = [
    { vendorId: 1234, classCode: 0xFF, serialNumber: "123ABC" },
    { vendorId: 5678, productId: 0xF00F }
  ];

  navigator.usb.test.onrequestdevice = event => {
    navigator.usb.test.onrequestdevice = undefined;

    assert_equals(event.filters.length, expectedFilters.length);
    for (var i = 0; i < event.filters.length; ++i) {
      assert_object_equals(event.filters[i], expectedFilters[i]);
    }

    event.respondWith(null);
  };

  return callWithTrustedClick(() => {
    return navigator.usb.requestDevice({ filters: expectedFilters })
      .then(device => {
        assert_unreachable('requestDevice should reject because no device selected');
      })
      .catch(error => {
        assert_equals(error.code, DOMException.NOT_FOUND_ERR);
      });
  });
}, 'filters are sent correctly');

usb_test(() => {
  return getFakeDevice().then(({ device, fakeDevice }) => {
    navigator.usb.test.onrequestdevice = event => {
      event.respondWith(fakeDevice);
    };
    return callWithTrustedClick(() => {
      let first = navigator.usb.requestDevice({ filters: [] });
      let second = navigator.usb.requestDevice({ filters: [] });
      return Promise.all([
        first.then(chosenDevice => {
          assert_equals(chosenDevice, device);
        }),
        second.then(chosenDevice => {
          assert_equals(chosenDevice, device);
        })
      ]);
    });
  });
}, 'multiple requestDevice calls are allowed per user activation');
