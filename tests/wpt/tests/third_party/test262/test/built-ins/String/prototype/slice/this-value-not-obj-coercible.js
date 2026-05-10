// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.slice
description: The "this" value must be object-coercible
info: |
  1. Let O be ? RequireObjectCoercible(this value).
---*/

var slice = String.prototype.slice;

assert.sameValue(typeof slice, 'function');

assert.throws(TypeError, function() {
  slice.call(undefined, 0);
}, 'undefined');

assert.throws(TypeError, function() {
  slice.call(null, 0);
}, 'null');
