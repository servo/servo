// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.clear
description: >
  Throws a TypeError if `this` is not an Object.
info: |
  Map.prototype.clear ( )

  1. Let M be the this value.
  2. If Type(M) is not Object, throw a TypeError exception.
  ...
features: [Symbol]
---*/

assert.throws(TypeError, function() {
  Map.prototype.clear.call(1);
});

assert.throws(TypeError, function() {
  Map.prototype.clear.call(true);
});

assert.throws(TypeError, function() {
  Map.prototype.clear.call('');
});

assert.throws(TypeError, function() {
  Map.prototype.clear.call(null);
});

assert.throws(TypeError, function() {
  Map.prototype.clear.call(undefined);
});

assert.throws(TypeError, function() {
  Map.prototype.clear.call(Symbol());
});
