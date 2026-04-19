// Copyright (C) 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-json.parse
description: >
  Context argument and its source property behave as expected when parsing an
  ObjectLiteral JSON string
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

assertOnlyOwnProperties(
  JSON.parse('{}', reviverWithExpectedSources([])),
  [],
  "empty object"
);

const singleProp = JSON.parse('{"42":37}', reviverWithExpectedSources(['37']));
assertOnlyOwnProperties(singleProp, ["42"], "single numeric property key");
assert.sameValue(singleProp[42], 37, "value of single numeric property key");

const multipleProps = JSON.parse('{"x": 1, "y": 2}', reviverWithExpectedSources(['1', '2']));
assertOnlyOwnProperties(multipleProps, ["x", "y"], "multiple properties");
assert.sameValue(multipleProps.x, 1, "multiple properties, value of x");
assert.sameValue(multipleProps.y, 2, "multiple properties, value of y");

// undefined means the json value is JSObject or JSArray and the passed
// context to the reviver function has no source property.
const arrayProps = JSON.parse(
  '{"x": [1,2], "y": [2,3]}',
  reviverWithExpectedSources(['1', '2', undefined, '2', '3', undefined])
);
assertOnlyOwnProperties(arrayProps, ["x", "y"], "array-valued properties");
assert.compareArray(arrayProps.x, [1, 2], "array-valued properties, value of x");
assert.compareArray(arrayProps.y, [2, 3], "array-valued properties, value of y");

const objectProps = JSON.parse(
  '{"x": {"x": 1, "y": 2}}',
  reviverWithExpectedSources(['1', '2', undefined, undefined])
);
assertOnlyOwnProperties(objectProps, ["x"], "object-valued properties");
assertOnlyOwnProperties(objectProps.x, ["x", "y"], "object-valued properties, value of x");
assert.sameValue(objectProps.x.x, 1, "object-valued properties, value of x.x");
assert.sameValue(objectProps.x.y, 2, "object-valued properties, value of x.y");
