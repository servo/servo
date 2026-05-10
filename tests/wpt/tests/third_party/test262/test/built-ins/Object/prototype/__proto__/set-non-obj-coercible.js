// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.prototype.__proto__
es6id: B.2.2.1
description: Called on a value that is not object-coercible
info: |
    1. Let O be ? RequireObjectCoercible(this value).
features: [__proto__]
---*/

var set = Object.getOwnPropertyDescriptor(Object.prototype, '__proto__').set;

assert.sameValue(typeof set, 'function');

assert.throws(TypeError, function() {
  set.call(undefined);
});

assert.throws(TypeError, function() {
  set.call(null);
});
