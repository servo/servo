// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.delete
description: >
  Throws a TypeError if `this` is not an Object.
info: |
  Map.prototype.delete ( key )

  1. Let M be the this value.
  2. If Type(M) is not Object, throw a TypeError exception.
  ...
features: [Symbol]
---*/

assert.throws(TypeError, function() {
  Map.prototype.delete.call(1, 'attr');
});

assert.throws(TypeError, function() {
  Map.prototype.delete.call(true, 'attr');
});

assert.throws(TypeError, function() {
  Map.prototype.delete.call('', 'attr');
});

assert.throws(TypeError, function() {
  Map.prototype.delete.call(null, 'attr');
});

assert.throws(TypeError, function() {
  Map.prototype.delete.call(undefined, 'attr');
});

assert.throws(TypeError, function() {
  Map.prototype.delete.call(Symbol(), 'attr');
});
