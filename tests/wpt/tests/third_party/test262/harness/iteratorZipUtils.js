// Copyright (C) 2025 André Bargull.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    Utility functions for testing Iterator.prototype.zip and Iterator.prototype.zipKeyed. Requires inclusion of propertyHelper.js.
defines:
  - forEachSequenceCombination
  - forEachSequenceCombinationKeyed
  - assertZipped
  - assertZippedKeyed
  - assertIteratorResult
  - assertIsPackedArray
---*/

// Assert |result| is an object created by CreateIteratorResultObject.
function assertIteratorResult(result, value, done, label) {
  assert.sameValue(
    Object.getPrototypeOf(result),
    Object.prototype,
    label + ": [[Prototype]] of iterator result is Object.prototype"
  );

  assert(Object.isExtensible(result), label + ": iterator result is extensible");

  var ownKeys = Reflect.ownKeys(result);
  assert.compareArray(ownKeys, ["value", "done"], label + ": iterator result properties");

  verifyProperty(result, "value", {
    value: value,
    writable: true,
    enumerable: true,
    configurable: true,
  });

  verifyProperty(result, "done", {
    value: done,
    writable: true,
    enumerable: true,
    configurable: true,
  });
}

// Assert |array| is a packed array with default property attributes.
function assertIsPackedArray(array, label) {
  assert(Array.isArray(array), label + ": array is an array exotic object");

  assert.sameValue(
    Object.getPrototypeOf(array),
    Array.prototype,
    label + ": [[Prototype]] of array is Array.prototype"
  );

  assert(Object.isExtensible(array), label + ": array is extensible");

  // Ensure "length" property has its default property attributes.
  verifyProperty(array, "length", {
    writable: true,
    enumerable: false,
    configurable: false,
  });

  // Ensure no holes and all elements have the default property attributes.
  for (var i = 0; i < array.length; i++) {
    verifyProperty(array, i, {
      writable: true,
      enumerable: true,
      configurable: true,
    });
  }
}

// Assert |array| is an extensible null-prototype object with default property attributes.
function _assertIsNullProtoMutableObject(object, label) {
  assert.sameValue(
    Object.getPrototypeOf(object),
    null,
    label + ": [[Prototype]] of object is null"
  );

  assert(Object.isExtensible(object), label + ": object is extensible");

  // Ensure all properties have the default property attributes.
  var keys = Object.getOwnPropertyNames(object);
  for (var i = 0; i < keys.length; i++) {
    verifyProperty(object, keys[i], {
      writable: true,
      enumerable: true,
      configurable: true,
    });
  }
}

// Assert that the `zipped` iterator yields the first `count` outputs of Iterator.zip.
// Assumes `inputs` is an array of arrays, each with length >= `count`.
// Advances `zipped` by `count` steps.
function assertZipped(zipped, inputs, count, label) {
  // Last returned elements array.
  var last = null;

  for (var i = 0; i < count; i++) {
    var itemLabel = label + ", step " + i;

    var result = zipped.next();
    var value = result.value;

    // Test IteratorResult structure.
    assertIteratorResult(result, value, false, itemLabel);

    // Ensure value is a new array.
    assert.notSameValue(value, last, itemLabel + ": returns a new array");
    last = value;

    // Ensure all array elements have the expected value.
    var expected = inputs.map(function (array) {
      return array[i];
    });
    assert.compareArray(value, expected, itemLabel + ": values");

    // Ensure value is a packed array with default data properties.
    assertIsPackedArray(value, itemLabel);
  }
}

// Assert that the `zipped` iterator yields the first `count` outputs of Iterator.zipKeyed.
// Assumes `inputs` is an object whose values are arrays, each with length >= `count`.
// Advances `zipped` by `count` steps.
function assertZippedKeyed(zipped, inputs, count, label) {
  // Last returned elements array.
  var last = null;

  var expectedKeys = Object.keys(inputs);

  for (var i = 0; i < count; i++) {
    var itemLabel = label + ", step " + i;

    var result = zipped.next();
    var value = result.value;

    // Test IteratorResult structure.
    assertIteratorResult(result, value, false, itemLabel);

    // Ensure resulting object is a new object.
    assert.notSameValue(value, last, itemLabel + ": returns a new object");
    last = value;

    // Ensure resulting object has the expected keys and values.
    assert.compareArray(Reflect.ownKeys(value), expectedKeys, itemLabel + ": result object keys");

    var expectedValues = Object.values(inputs).map(function (array) {
      return array[i];
    });
    assert.compareArray(Object.values(value), expectedValues, itemLabel + ": result object values");

    // Ensure resulting object is a null-prototype mutable object with default data properties.
    _assertIsNullProtoMutableObject(value, itemLabel);
  }
}

function forEachSequenceCombinationKeyed(callback) {
  return forEachSequenceCombination(function(inputs, inputsLabel, min, max) {
    var object = {};
    for (var i = 0; i < inputs.length; ++i) {
      object["prop_" + i] = inputs[i];
    }
    inputsLabel = "inputs = " + JSON.stringify(object);
    callback(object, inputsLabel, min, max);
  });
}

function forEachSequenceCombination(callback) {
  function test(inputs) {
    if (inputs.length === 0) {
      callback(inputs, "inputs = []", 0, 0);
      return;
    }

    var lengths = inputs.map(function(array) {
      return array.length;
    });

    var min = Math.min.apply(null, lengths);
    var max = Math.max.apply(null, lengths);

    var inputsLabel = "inputs = " + JSON.stringify(inputs);

    callback(inputs, inputsLabel, min, max);
  }

  // Yield all prefixes of the string |s|.
  function* prefixes(s) {
    for (var i = 0; i <= s.length; ++i) {
      yield s.slice(0, i);
    }
  }

  // Zip an empty iterable.
  test([]);

  // Zip a single iterator.
  for (var prefix of prefixes("abcd")) {
    test([prefix.split("")]);
  }

  // Zip two iterators.
  for (var prefix1 of prefixes("abcd")) {
    for (var prefix2 of prefixes("efgh")) {
      test([prefix1.split(""), prefix2.split("")]);
    }
  }

  // Zip three iterators.
  for (var prefix1 of prefixes("abcd")) {
    for (var prefix2 of prefixes("efgh")) {
      for (var prefix3 of prefixes("ijkl")) {
        test([prefix1.split(""), prefix2.split(""), prefix3.split("")]);
      }
    }
  }
}
