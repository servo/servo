export function assertEqualDates(actual, expected, label) {
  assert_equals(expected.constructor, Date,
    "assertEqualDates usage check: expected must be a Date");

  const labelPart = label === undefined ? "" : `${label}: `;
  assert_equals(actual.constructor, Date, `${labelPart}must be a Date`);
  assert_equals(actual.valueOf(), expected.valueOf(), `${labelPart}timestamps must match`);
}

export function assertEqualArrayBuffers(actual, expected, label) {
  assert_equals(expected.constructor, ArrayBuffer,
    "assertEqualArrayBuffers usage check: expected must be an ArrayBuffer");

  const labelPart = label === undefined ? "" : `${label}: `;
  assert_equals(actual.constructor, ArrayBuffer, `${labelPart}must be an ArrayBuffer`);
  assert_array_equals(new Uint8Array(actual), new Uint8Array(expected), `${labelPart}must match`);
}

export function assertArrayBufferEqualsABView(actual, expected, label) {
  assert_true(ArrayBuffer.isView(expected),
    "assertArrayBufferEqualsABView usage check: expected must be an ArrayBuffer view");

  assertEqualArrayBuffers(actual, expected.buffer, label);
}

export function assertArrayCustomEquals(actual, expected, equalityAsserter, label) {
  assert_true(Array.isArray(expected),
    "assertArrayCustomEquals usage check: expected must be an Array");

  const labelPart = label === undefined ? "" : `${label}: `;
  assert_true(Array.isArray(actual), `${labelPart}must be an array`);
  assert_equals(actual.length, expected.length, `${labelPart}length must be as expected`);

  for (let i = 0; i < actual.length; ++i) {
    equalityAsserter(actual[i], expected[i], `${labelPart}index ${i}`);
  }
}
