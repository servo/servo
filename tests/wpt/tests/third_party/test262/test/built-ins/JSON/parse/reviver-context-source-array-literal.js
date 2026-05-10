// Copyright (C) 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-json.parse
description: >
  Context argument and its source property behave as expected when parsing an
  ArrayLiteral JSON string
includes: [compareArray.js, propertyHelper.js]
features: [json-parse-with-source]
---*/

function assertOnlyOwnProperties(object, props, message) {
  assert.compareArray(Object.getOwnPropertyNames(object), props, `${message}: object should have no other properties than expected`);
  assert.compareArray(Object.getOwnPropertySymbols(object), [], `${message}: object should have no own symbol properties`);
}

function reviverWithExpectedSources(expectedSources) {
  let i = 0;
  return function reviver(key, value, context) {
    assert.sameValue(typeof context, "object", "context should be an object");
    assert.sameValue(Object.getPrototypeOf(context), Object.prototype, "context should be a plain object");
    if (expectedSources[i] !== undefined) {
      assertOnlyOwnProperties(context, ["source"],
        "the JSON value is a primitve value, its context should only have a source property");
      verifyProperty(context, "source", {
        value: expectedSources[i++],
        configurable: true,
        enumerable: true,
        writable: true,
      }, { restore: true });
    } else {
      assertOnlyOwnProperties(context, [],
        "the JSON value is an Array or Object, its context should have no property");
      i++;
    }
    return value;
  };
}

assert.compareArray(JSON.parse('[1.0]', reviverWithExpectedSources(['1.0'])), [1]);
assert.compareArray(
  JSON.parse('[1.1]', reviverWithExpectedSources(['1.1'])),
  [1.1]
);
assert.compareArray(JSON.parse('[]', reviverWithExpectedSources([])), []);

const longArray = JSON.parse(
  '[1, "2", true, null, {"x": 1, "y": 1}]',
  reviverWithExpectedSources(['1', '"2"', 'true', 'null', '1', '1'])
);
assert.sameValue(longArray[0], 1, "array, element 0");
assert.sameValue(longArray[1], "2", "array, element 1");
assert.sameValue(longArray[2], true, "array, element 2");
assert.sameValue(longArray[3], null, "array, element 3");
assertOnlyOwnProperties(longArray[4], ["x", "y"], "array, element 5");
assert.sameValue(longArray[4].x, 1, "array, element 5, prop x");
assert.sameValue(longArray[4].y, 1, "array, element 5, prop y");
