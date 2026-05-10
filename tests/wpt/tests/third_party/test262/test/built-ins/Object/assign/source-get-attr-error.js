// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.1
description: Errors thrown during retrieval of source object attributes
info: |
    [...]
    5. For each element nextSource of sources, in ascending index order,
    [...]
    c. Repeat for each element nextKey of keys in List order,
       [...]
       iii. if desc is not undefined and desc.[[Enumerable]] is true, then
            1. Let propValue be Get(from, nextKey).
            2. ReturnIfAbrupt(propValue).
---*/

var source = {
  get attr() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  Object.assign({}, source);
});
