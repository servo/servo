// META: script=/resources/testharness.js
// META: script=/resources/testharnessreport.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=/bluetooth/resources/bluetooth-helpers.js
'use strict';
const test_desc = `A read request succeeds and returns the descriptor's value.`;

bluetooth_test(async () => {
  const {descriptor, fake_descriptor} = await getUserDescriptionDescriptor();

  const EXPECTED_VALUE = [0, 1, 2];
  await fake_descriptor.setNextReadResponse(GATT_SUCCESS, EXPECTED_VALUE);

  const value = await descriptor.readValue();
  assert_array_equals(Array.from(new Uint8Array(value.buffer)), EXPECTED_VALUE);
}, test_desc);
