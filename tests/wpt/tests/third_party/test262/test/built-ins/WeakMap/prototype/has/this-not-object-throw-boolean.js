// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.has
description: Throws TypeError if `this` is not Object.
info: |
  WeakMap.prototype.has ( value )

  1. Let S be the this value.
  2. If Type(S) is not Object, throw a TypeError exception.

---*/

assert.throws(TypeError, function() {
  WeakMap.prototype.has.call(false, {});
});

assert.throws(TypeError, function() {
  var map = new WeakMap();
  map.has.call(false, {});
});
