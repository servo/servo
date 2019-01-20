export function assertEqualDates(actual, expected, label) {
  label = formatLabel(label);

  assert_equals(expected.constructor, Date,
    `${label}assertEqualDates usage check: expected must be a Date`);

  assert_equals(actual.constructor, Date, `${label}must be a Date`);
  assert_equals(actual.valueOf(), expected.valueOf(), `${label}timestamps must match`);
}

export function assertEqualPostKeyRoundtripping(actual, expected, label) {
  label = formatLabel(label);

  // Please extend this to support other types as needed!
  assert_true(
    typeof expected === "number" || typeof expected === "string" || expected.constructor === Date,
    `${label}assertEqualPostKeyRoundtripping usage check: currently only supports numbers, strings, and dates`
  );

  if (expected.constructor === Date) {
    assert_equals(actual.constructor, Date, `${label}comparing to Date(${Number(expected)}) (actual = ${actual})`);
    actual = Number(actual);
    expected = Number(expected);
  }

  assert_equals(actual, expected, label);
}

export function assertEqualArrayBuffers(actual, expected, label) {
  label = formatLabel(label);

  assert_equals(expected.constructor, ArrayBuffer,
    `${label}assertEqualArrayBuffers usage check: expected must be an ArrayBuffer`);

  assert_equals(actual.constructor, ArrayBuffer, `${label}must be an ArrayBuffer`);
  assert_array_equals(new Uint8Array(actual), new Uint8Array(expected), `${label}must match`);
}

export function assertArrayBufferEqualsABView(actual, expected, label) {
  label = formatLabel(label);

  assert_true(ArrayBuffer.isView(expected),
    `${label}assertArrayBufferEqualsABView usage check: expected must be an ArrayBuffer view`);

  assertEqualArrayBuffers(actual, expected.buffer, label);
}

export function assertAsyncIteratorEquals(actual, expected, label) {
  return assertAsyncIteratorCustomEquals(actual, expected, Object.is, label);
}

export function assertArrayCustomEquals(actual, expected, equalityAsserter, label) {
  label = formatLabel(label);

  assert_true(Array.isArray(expected),
    `${label} assertArrayCustomEquals usage check: expected must be an Array`);

  assert_true(Array.isArray(actual), `${label}must be an array`);
  assert_equals(actual.length, expected.length, `${label}length must be as expected`);

  for (let i = 0; i < actual.length; ++i) {
    equalityAsserter(actual[i], expected[i], `${label}index ${i}`);
  }
}

export async function assertAsyncIteratorCustomEquals(actual, expected, equalityAsserter, label) {
  label = formatLabel(label);

  assert_true(Array.isArray(expected),
    `${label} assertAsyncIteratorCustomEquals usage check: expected must be an Array`);

  const collected = await collectAsyncIterator(actual);
  assert_equals(collected.length, expected.length, `${label}length must be as expected`);

  for (let i = 0; i < collected.length; ++i) {
    equalityAsserter(collected[i], expected[i], `${label}index ${i}`);
  }
}

async function collectAsyncIterator(asyncIterator) {
  const array = [];
  for await (const entry of asyncIterator) {
    array.push(entry);
  }

  return array;
}

function formatLabel(label) {
  return label !== undefined ? `${label} ` : "";
}
