// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

var F, o;

F = function () {};
F.prototype = new ArrayBuffer(1);
o = new F();
assert.throws(TypeError, function() {
  o.byteLength;
});

o = {};
o.__proto__ = new Int32Array(1);
assert.throws(TypeError, function() {
  o.buffer.byteLength;
});

F = function () {};
F.prototype = new Int32Array(1);
o = new F();
assert.throws(TypeError, function() {
  o.slice(0, 1);
});
