// Copyright (C) 2018 Shilpi Jain and Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.flat
description: >
    using bound functions
includes: [compareArray.js]
features: [Array.prototype.flat]
---*/

var a = [
  [0],
  [1]
];
var actual = [].flat.bind(a)();

assert.compareArray(actual, [0, 1], 'The value of actual is expected to be [0, 1]');
