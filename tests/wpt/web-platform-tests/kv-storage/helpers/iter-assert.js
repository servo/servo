export function iterResultCustom(o, expectedValue, expectedDone, valueAsserter, label) {
  label = formatLabel(label);

  assert_equals(typeof expectedDone, "boolean",
    `${label} iterResult assert usage check: expectedDone must be a boolean`);

  propertyKeys(o, ["value", "done"], [], label);
  assert_equals(Object.getPrototypeOf(o), Object.prototype, `${label}prototype must be Object.prototype`);
  valueAsserter(o.value, expectedValue, `${label}value`);
  assert_equals(o.done, expectedDone, `${label}done`);
}

export function iterResult(o, expectedValue, expectedDone, label) {
  return iterResultCustom(o, expectedValue, expectedDone, assert_equals, label);
}

export function iterResultsCustom(actualArray, expectedArrayOfArrays, valueAsserter, label) {
  label = formatLabel(label);

  assert_equals(actualArray.length, expectedArrayOfArrays.length,
    `${label} iterResults assert usage check: actual and expected must have the same length`);

  for (let i = 0; i < actualArray.length; ++i) {
    const [expectedValue, expectedDone] = expectedArrayOfArrays[i];
    iterResultCustom(actualArray[i], expectedValue, expectedDone, valueAsserter, `${label}iter result ${i}`);
  }
}

export function iterResults(actualArray, expectedArrayOfArrays, label) {
  return iterResultsCustom(actualArray, expectedArrayOfArrays, assert_equals, label);
}

function propertyKeys(o, expectedNames, expectedSymbols, label) {
  label = formatLabel(label);
  assert_array_equals(Object.getOwnPropertyNames(o), expectedNames, `${label}property names`);
  assert_array_equals(Object.getOwnPropertySymbols(o), expectedSymbols,
    `${label}property symbols`);
}

function formatLabel(label) {
  return label !== undefined ? `${label} ` : "";
}
