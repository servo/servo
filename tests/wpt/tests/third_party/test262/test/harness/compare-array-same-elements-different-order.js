// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Arrays containg the same elements in different order are not equivalent.
includes: [compareArray.js]
---*/

var obj = {};
var first = [0, 1, '', 's', null, undefined, obj];
var second = [0, 1, '', 's', undefined, null, obj];

assert.throws(Test262Error, () => {
  assert.compareArray(first, second);
}, 'Arrays containing the same elements in different order are not equivalent.');
