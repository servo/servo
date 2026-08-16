// Copyright (C) 2026 Ojus Chugh. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype-@@unscopables
description: >
    Array.prototype[Symbol.unscopables].at is true
info: |
    22.1.3.32 Array.prototype [ @@unscopables ]

    ...
    2. Perform ! CreateDataPropertyOrThrow(unscopableList, "at", true).
    ...

includes: [propertyHelper.js]
features: [Symbol.unscopables, Array.prototype.at]
---*/

var unscopables = Array.prototype[Symbol.unscopables];

assert.sameValue(unscopables.at, true, '`at` property value');
verifyProperty(unscopables, "at", {
  writable: true,
  enumerable: true,
  configurable: true
});
