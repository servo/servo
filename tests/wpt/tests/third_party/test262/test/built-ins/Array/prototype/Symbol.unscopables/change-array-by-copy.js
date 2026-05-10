// Copyright (C) 2021 Igalia. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype-@@unscopables
description: >
    Initial value of `Symbol.unscopables` property
info: |
    22.1.3.32 Array.prototype [ @@unscopables ]

    ...
    12. Perform ! CreateDataPropertyOrThrow(unscopableList, "toReversed", true).
    13. Perform ! CreateDataPropertyOrThrow(unscopableList, "toSorted", true).
    14. Perform ! CreateDataPropertyOrThrow(unscopableList, "toSpliced", true).
    ...
includes: [propertyHelper.js]
features: [Symbol.unscopables, change-array-by-copy]
---*/

var unscopables = Array.prototype[Symbol.unscopables];

for (const unscopable of ["toReversed", "toSorted", "toSpliced"]) {
    verifyProperty(unscopables, unscopable, {
        value: true,
        writable: true,
        configurable: true
    })
};

assert(!Object.prototype.hasOwnProperty.call(unscopables, "with"), "does not have `with`");
