// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.1
description: Invoked with a source whose own property descriptor cannot be retrieved
info: |
    [...]
    5. For each element nextSource of sources, in ascending index order,
       [...]
       c. Repeat for each element nextKey of keys in List order,
          i. Let desc be from.[[GetOwnProperty]](nextKey).
          ii. ReturnIfAbrupt(desc).
features: [Proxy]
---*/

var source = new Proxy({
  attr: null
}, {
  getOwnPropertyDescriptor: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  Object.assign({}, source);
});
