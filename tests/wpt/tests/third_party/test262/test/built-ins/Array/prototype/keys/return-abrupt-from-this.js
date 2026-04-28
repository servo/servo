// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.keys
description: >
  Return abrupt from ToObject(this value).
info: |
  22.1.3.13 Array.prototype.keys ( )

  1. Let O be ToObject(this value).
  2. ReturnIfAbrupt(O).
---*/

assert.throws(TypeError, function() {
  Array.prototype.keys.call(undefined);
});

assert.throws(TypeError, function() {
  Array.prototype.keys.call(null);
});
