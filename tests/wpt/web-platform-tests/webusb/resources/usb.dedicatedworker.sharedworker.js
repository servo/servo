'use strict';
importScripts('/resources/testharness.js');

test(() => {
  assert_true(navigator.usb instanceof USB,
      'navigator.usb should be defined as a USB object');
  assert_true(navigator.usb.getDevices instanceof Function,
      'navigator.usb.getDevices should be defined as a function');
  assert_equals(typeof navigator.usb.requestDevice, 'undefined',
      'navigator.usb.requestDevice should not be defined');
}, 'Web workers should have the WebUSB API exposed as defined in the spec.');

done();