// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.concat
description: The "this" value must be object-coercible
info: |
  1. Let O be ? RequireObjectCoercible(this value).
---*/

var concat = String.prototype.concat;

assert.sameValue(typeof concat, 'function');

assert.throws(TypeError, function() {
  concat.call(undefined, '');
}, 'undefined');

assert.throws(TypeError, function() {
  concat.call(null, '');
}, 'null');
