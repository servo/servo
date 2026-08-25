// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.1
description: Invoked with a source whose own property keys cannot be retrieved
info: |
    [...]
    5. For each element nextSource of sources, in ascending index order,
       a. If nextSource is undefined or null, let keys be an empty List.
       b. Else,
          i. Let from be ToObject(nextSource).
          ii. ReturnIfAbrupt(from).
          iii. Let keys be from.[[OwnPropertyKeys]]().
          iv. ReturnIfAbrupt(keys).
features: [Proxy]
---*/

var source = new Proxy({}, {
  ownKeys: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  Object.assign({}, source);
});
