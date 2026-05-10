// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.1
description: >
    arrow functions
includes: [compareArray.js]
---*/
var fn = (a, b, ...c) => c;

assert.compareArray(fn(), []);
assert.compareArray(fn(1, 2), []);
assert.compareArray(fn(1, 2, 3), [3]);
assert.compareArray(fn(1, 2, 3, 4), [3, 4]);
assert.compareArray(fn(1, 2, 3, 4, 5), [3, 4, 5]);
assert.compareArray(((...args) => args)(), []);
assert.compareArray(((...args) => args)(1,2,3), [1,2,3]);
