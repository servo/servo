// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

// The Device Orientation spec does not fully integrate with the Permissions
// spec and does not list the permissions that are expected for
// requestPermission() to work. The list below was based on the permissions
// listed in https://w3c.github.io/orientation-sensor/#model for the low-level
// sensors that power absolute and relative orientation sensors.
const permissionDescriptorNames =
    ['accelerometer', 'gyroscope', 'magnetometer'];

promise_test(async (t) => {
  await Promise.all(permissionDescriptorNames.map(
      name => test_driver.set_permission({name}, 'granted')));

  const permission = await DeviceOrientationEvent.requestPermission();
  assert_equals(permission, 'granted');
}, 'requestPermission() returns "granted" for granted permissions without user activation');

promise_test(async (t) => {
  await Promise.all(permissionDescriptorNames.map(
      name => test_driver.set_permission({name}, 'granted')));

  return test_driver.bless('enable user activation', async () => {
    const permission = await DeviceOrientationEvent.requestPermission();
    assert_equals(permission, 'granted');
  });
}, 'requestPermission() returns "granted" for granted permissions with user activation');

promise_test(async (t) => {
  await Promise.all(permissionDescriptorNames.map(
      name => test_driver.set_permission({name}, 'denied')));

  const permission = await DeviceOrientationEvent.requestPermission();
  assert_equals(permission, 'denied');
}, 'requestPermission() returns "denied" for denied permissions without user activation');

promise_test(async (t) => {
  await Promise.all(permissionDescriptorNames.map(
      name => test_driver.set_permission({name}, 'denied')));

  return test_driver.bless('enable user activation', async () => {
    const permission = await DeviceOrientationEvent.requestPermission();
    assert_equals(permission, 'denied');
  });
}, 'requestPermission() returns "denied" for denied permissions with user activation');
