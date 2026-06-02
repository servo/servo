// Compare two DataViews byte-by-byte.
function compareDataViews(actual, expected) {
  assert_true(actual instanceof DataView, 'actual is DataView');
  assert_true(expected instanceof DataView, 'expected is DataView');
  assert_equals(actual.byteLength, expected.byteLength, 'lengths equal');
  for (let i = 0; i < expected.byteLength; ++i) {
    assert_equals(
        actual.getUint8(i), expected.getUint8(i), `Mismatch at byte ${i}.`);
  }
}

// Returns a Promise that resolves once |device| receives an input report.
function oninputreport(device) {
  assert_true(device instanceof HIDDevice);
  return new Promise(resolve => {
    device.oninputreport = resolve;
  });
}
