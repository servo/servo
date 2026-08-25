// Copyright (C) 2025 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.zipkeyed
description: >
  Returned object has the correct prototype and default property attributes.
info: |
  Iterator.zipKeyed ( iterables [ , options ] )
    ...
    15. Let finishResults be a new Abstract Closure with parameters (results) that captures keys and iterCount and performs the following steps when called:
      a. Let obj be OrdinaryObjectCreate(null).
      b. For each integer i such that 0 ≤ i < iterCount, in ascending order, do
        i. Perform ! CreateDataPropertyOrThrow(obj, keys[i], results[i]).
      c. Return obj.
    ...
includes: [compareArray.js, propertyHelper.js]
features: [joint-iteration]
---*/

// Assert |actual| is a plain object equal to |expected| with default property attributes.
function assertPlainObject(actual, expected) {
  assert.sameValue(
    Object.getPrototypeOf(actual),
    null,
    "[[Prototype]] of actual is null"
  );

  assert(Object.isExtensible(actual), "actual is extensible");

  var actualKeys = Reflect.ownKeys(actual);
  var expectedKeys = Reflect.ownKeys(expected);

  // All expected property keys are present.
  assert.compareArray(actualKeys, expectedKeys);

  // All expected property values are equal.
  for (var key of expectedKeys) {
    assert.sameValue(actual[key], expected[key], "with key: " + String(key));
  }

  // Ensure all properties have the default property attributes.
  for (var key of expectedKeys) {
    verifyProperty(actual, key, {
      writable: true,
      enumerable: true,
      configurable: true,
    });
  }
}

var iterables = Object.create(null, {
  a: {
    writable: true,
    enumerable: true,
    configurable: true,
    value: ["A"],
  },
  b: {
    writable: false,
    enumerable: true,
    configurable: true,
    value: ["B"],
  },
  c: {
    writable: true,
    enumerable: true,
    configurable: false,
    value: ["C"],
  },
  d: {
    writable: false,
    enumerable: true,
    configurable: false,
    value: ["D"],
  },
  e: {
    enumerable: true,
    configurable: true,
    get() {
      return ["E"];
    }
  },
  f: {
    enumerable: true,
    configurable: false,
    get() {
      return ["F"];
    }
  },
});

var it = Iterator.zipKeyed(iterables);

var results = it.next().value;

assertPlainObject(results, {
  a: "A",
  b: "B",
  c: "C",
  d: "D",
  e: "E",
  f: "F",
});

assert.sameValue(it.next().done, true, "iterator is exhausted");
