// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.13
description: >
  Return abrupt from ToPropertyKey(propertyKey)
info: |
  26.1.13 Reflect.set ( target, propertyKey, V [ , receiver ] )

  ...
  2. Let key be ToPropertyKey(propertyKey).
  3. ReturnIfAbrupt(key).
  ...
features: [Reflect, Reflect.set]
---*/

var p = {
  toString: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  Reflect.set({}, p);
});
