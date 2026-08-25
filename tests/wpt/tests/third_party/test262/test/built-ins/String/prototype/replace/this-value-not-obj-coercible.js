// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.replace
description: The "this" value must be object-coercible
info: |
  1. Let O be ? RequireObjectCoercible(this value).
---*/

var replace = String.prototype.replace;

assert.sameValue(typeof replace, 'function');

assert.throws(TypeError, function() {
  replace.call(undefined, '', '');
}, 'undefined');

assert.throws(TypeError, function() {
  replace.call(null, '', '');
}, 'null');
