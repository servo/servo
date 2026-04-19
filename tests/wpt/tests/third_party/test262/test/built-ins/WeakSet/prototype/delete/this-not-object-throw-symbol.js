// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakset.prototype.delete
description: Throws TypeError if `this` is not Object.
info: |
  WeakSet.prototype.delete ( value )

  1. Let S be the this value.
  2. If Type(S) is not Object, throw a TypeError exception.
features: [Symbol]
---*/

assert.throws(TypeError, function() {
  WeakSet.prototype.delete.call(Symbol(), {});
});

assert.throws(TypeError, function() {
  var s = new WeakSet();
  s.delete.call(Symbol(), {});
});
