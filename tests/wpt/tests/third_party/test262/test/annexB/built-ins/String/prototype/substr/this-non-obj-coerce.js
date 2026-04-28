// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.substr
es6id: B.2.3.1
description: Behavior when "this" value is not Object-coercible
info: |
    1. Let O be ? RequireObjectCoercible(this value).
---*/

var substr = String.prototype.substr;

assert.sameValue(typeof substr, 'function');

assert.throws(TypeError, function() {
  substr.call(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  substr.call(null);
}, 'null');
