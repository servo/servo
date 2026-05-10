// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.4-4-49
description: >
    Object.getOwnPropertyNames - own index properties of Array objcect
    are pushed into the returned Array
includes: [compareArray.js]
---*/

var arr = [0, 1, 2];
assert.compareArray(Object.getOwnPropertyNames(arr), ["0", "1", "2", "length"]);
