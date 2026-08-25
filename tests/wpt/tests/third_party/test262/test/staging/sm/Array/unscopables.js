// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js, propertyHelper.js]
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
let Array_unscopables = Array.prototype[Symbol.unscopables];

verifyProperty(Array.prototype, Symbol.unscopables, {
    value: Array_unscopables,
    writable: false,
    enumerable: false,
    configurable: true
}, {
    restore: true
});

assert.sameValue(Reflect.getPrototypeOf(Array_unscopables), null);

verifyProperty(Array_unscopables, "values", {
    value: true,
    writable: true,
    enumerable: true,
    configurable: true
}, {
    restore: true
});

let keys = Reflect.ownKeys(Array_unscopables);

let expectedKeys = [
    "at",
    "copyWithin",
    "entries",
    "fill",
    "find",
    "findIndex",
    "findLast",
    "findLastIndex",
    "flat",
    "flatMap",
    "includes",
    "keys",
    "toReversed",
    "toSorted",
    "toSpliced",
    "values"
];

assert.compareArray(keys, expectedKeys);

for (let key of keys)
    assert.sameValue(Array_unscopables[key], true);

// Test that it actually works
assert.throws(ReferenceError, () => {
    with ([]) {
        return entries;
    }
});

{
    let fill = 33;
    with (Array.prototype) {
        assert.sameValue(fill, 33);
    }
}

