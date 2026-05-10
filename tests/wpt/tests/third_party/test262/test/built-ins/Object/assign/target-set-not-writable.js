// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.1
description: Errors thrown during definition of target object attributes
info: |
    [...]
    5. For each element nextSource of sources, in ascending index order,
    [...]
    c. Repeat for each element nextKey of keys in List order,
       [...]
       iii. if desc is not undefined and desc.[[Enumerable]] is true, then
            [...]
            3. Let status be Set(to, nextKey, propValue, true).
            4. ReturnIfAbrupt(status).
---*/

var target = {};
Object.defineProperty(target, 'attr', {
  writable: false
});

assert.throws(TypeError, function() {
  Object.assign(target, {
    attr: 1
  });
});
