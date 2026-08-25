// Copyright (C) 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-json.parse
description: Codepaths involving InternaliseJSONProperty behave as expected

includes: [compareArray.js]
features: [json-parse-with-source]
---*/

function assertOnlyOwnProperties(object, props, message) {
  assert.compareArray(Object.getOwnPropertyNames(object), props, `${message}: object should have no other properties than expected`);
  assert.compareArray(Object.getOwnPropertySymbols(object), [], `${message}: object should have no own symbol properties`);
}

const replacements = [
  42,
  ["foo"],
  { foo: "bar" },
  "foo"
];

// Test Array forward modify
for (const replacement of replacements) {
  let alreadyReplaced = false;
  let expectedKeys = ["0", "1", ""];
  // if the replacement is an object, add its keys to the expected keys
  if (typeof replacement === "object") {
    expectedKeys.splice(1, 0, ...Object.keys(replacement));
  }
  const o = JSON.parse("[1, 2]", function (k, v, { source }) {
    assert.sameValue(k, expectedKeys.shift());
    if (k === "0") {
      if (!alreadyReplaced) {
        this[1] = replacement;
        alreadyReplaced = true;
      }
    } else if (k !== "") {
      assert.sameValue(source, undefined);
    }
    return this[k];
  });
  assert.sameValue(expectedKeys.length, 0);
  assert.compareArray(o, [1, replacement], `array forward-modified with ${replacement}`);
}

function assertOnlyOwnProperties(object, props, message) {
  assert.compareArray(Object.getOwnPropertyNames(object), props, `${message}: object should have no other properties than expected`);
  assert.compareArray(Object.getOwnPropertySymbols(object), [], `${message}: object should have no own symbol properties`);
}

// Test Object forward modify
for (const replacement of replacements) {
  let alreadyReplaced = false;
  let expectedKeys = ["p", "q", ""];
  if (typeof replacement === "object") {
    expectedKeys.splice(1, 0, ...Object.keys(replacement));
  }
  const o = JSON.parse('{"p":1, "q":2}', function (k, v, { source }) {
    assert.sameValue(k, expectedKeys.shift());
    if (k === 'p') {
      if (!alreadyReplaced) {
        this.q = replacement;
        alreadyReplaced = true;
      }
    } else if (k !== "") {
      assert.sameValue(source, undefined);
    }
    return this[k];
  });
  assert.sameValue(expectedKeys.length, 0);
  assertOnlyOwnProperties(o, ["p", "q"], `object forward-modified with ${replacement}`);
  assert.sameValue(o.p, 1, "property p should not be replaced");
  assert.sameValue(o.q, replacement, `property q should be replaced with ${replacement}`);
}

// Test combinations of possible JSON input with multiple forward modifications

{
  let reviverCallIndex = 0;
  const expectedKeys = ["a", "b", "c", ""];
  const reviver = function(key, value, {source}) {
    assert.sameValue(key, expectedKeys[reviverCallIndex++]);
    if (key === "a") {
      this.b = 2;
      assert.sameValue(source, "0");
    } else if (key === "b") {
      this.c = 3;
      assert.sameValue(value, 2);
      assert.sameValue(source, undefined);
    } else if (key === "c") {
      assert.sameValue(value, 3);
      assert.sameValue(source, undefined);
    }
    return value;
  }
  const parsed = JSON.parse('{"a": 0, "b": 1, "c": [1, 2]}', reviver);
  assertOnlyOwnProperties(parsed, ["a", "b", "c"], "object with forward-modified properties");
  assert.sameValue(parsed.a, 0, "'a' property should be unmodified");
  assert.sameValue(parsed.b, 2, "'b' property should be modified to 2");
  assert.sameValue(parsed.c, 3, "'c' property should be modified to 3");
}

{
  let reviverCallIndex = 0;
  const expectedKeys = ["0", "1", "2", "3", ""];
  const reviver = function(key, value, {source}) {
    assert.sameValue(key, expectedKeys[reviverCallIndex++]);
    if (key === "0") {
      this[1] = 3;
      assert.sameValue(value, 1);
      assert.sameValue(source, "1");
    } else if (key === "1") {
      this[2] = 4;
      assert.sameValue(value, 3);
      assert.sameValue(source, undefined);
    } else if(key === "2") {
      this[3] = 5;
      assert.sameValue(value, 4);
      assert.sameValue(source, undefined);
    } else if(key === "5") {
      assert.sameValue(value, 5);
      assert.sameValue(source, undefined);
    }
    return value;
  }
  assert.compareArray(JSON.parse('[1, 2, 3, {"a": 1}]', reviver), [1, 3, 4, 5], "array with forward-modified elements");
}
