// Copyright (C) 2015 Mike Pennisi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype-@@unscopables
description: >
    Initial value of `Symbol.unscopables` property
info: |
    22.1.3.32 Array.prototype [ @@unscopables ]

    1. Let unscopableList be ObjectCreate(null).
    2. Perform CreateDataProperty(unscopableList, "copyWithin", true).
    3. Perform CreateDataProperty(unscopableList, "entries", true).
    4. Perform CreateDataProperty(unscopableList, "fill", true).
    5. Perform CreateDataProperty(unscopableList, "find", true).
    6. Perform CreateDataProperty(unscopableList, "findIndex", true).
    7. Perform CreateDataProperty(unscopableList, "flat", true).
    8. Perform CreateDataProperty(unscopableList, "flatMap", true).
    9. Perform CreateDataProperty(unscopableList, "includes", true).
    10. Perform CreateDataProperty(unscopableList, "keys", true).
    11. Perform CreateDataProperty(unscopableList, "values", true).
    12. Assert: Each of the above calls returns true.
    13. Return unscopableList.

includes: [propertyHelper.js]
features: [Symbol.unscopables]
---*/

var unscopables = Array.prototype[Symbol.unscopables];

assert.sameValue(Object.getPrototypeOf(unscopables), null);

assert.sameValue(unscopables.copyWithin, true, '`copyWithin` property value');
verifyProperty(unscopables, "copyWithin", {
  writable: true,
  enumerable: true,
  configurable: true
});

assert.sameValue(unscopables.entries, true, '`entries` property value');
verifyProperty(unscopables, "entries", {
  writable: true,
  enumerable: true,
  configurable: true
});

assert.sameValue(unscopables.fill, true, '`fill` property value');
verifyProperty(unscopables, "fill", {
  writable: true,
  enumerable: true,
  configurable: true
});

assert.sameValue(unscopables.find, true, '`find` property value');
verifyProperty(unscopables, "find", {
  writable: true,
  enumerable: true,
  configurable: true
});

assert.sameValue(unscopables.findIndex, true, '`findIndex` property value');
verifyProperty(unscopables, "findIndex", {
  writable: true,
  enumerable: true,
  configurable: true
});

assert.sameValue(unscopables.flat, true, '`flat` property value');
verifyProperty(unscopables, "flat", {
  writable: true,
  enumerable: true,
  configurable: true
});

assert.sameValue(unscopables.flatMap, true, '`flatMap` property value');
verifyProperty(unscopables, "flatMap", {
  writable: true,
  enumerable: true,
  configurable: true
});

assert.sameValue(unscopables.includes, true, '`includes` property value');
verifyProperty(unscopables, "includes", {
  writable: true,
  enumerable: true,
  configurable: true
});

assert.sameValue(unscopables.keys, true, '`keys` property value');
verifyProperty(unscopables, "keys", {
  writable: true,
  enumerable: true,
  configurable: true
});

assert.sameValue(unscopables.values, true, '`values` property value');
verifyProperty(unscopables, "values", {
  writable: true,
  enumerable: true,
  configurable: true
});
