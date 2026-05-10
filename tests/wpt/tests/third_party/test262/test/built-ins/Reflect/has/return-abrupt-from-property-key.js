// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.9
description: >
  Return abrupt from ToPropertyKey(propertyKey)
info: |
  26.1.9 Reflect.has ( target, propertyKey )

  ...
  2. Let key be ToPropertyKey(propertyKey).
  3. ReturnIfAbrupt(key).
  ...
features: [Reflect]
---*/

var p = {
  toString: function() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  Reflect.has({}, p);
});
