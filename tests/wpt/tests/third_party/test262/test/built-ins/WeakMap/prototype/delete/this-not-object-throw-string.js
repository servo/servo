// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-weakmap.prototype.delete
description: Throws TypeError if `this` is not Object.
info: |
  WeakMap.prototype.delete ( value )

  1. Let M be the this value.
  2. If Type(M) is not Object, throw a TypeError exception.
---*/

assert.throws(TypeError, function() {
  WeakMap.prototype.delete.call('', {});
});

assert.throws(TypeError, function() {
  var map = new WeakMap();
  map.delete.call('', {});
});
