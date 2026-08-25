// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.trimend
description: The "this" value must be object-coercible
info: |
  1. Let O be ? RequireObjectCoercible(this value).
features: [string-trimming, String.prototype.trimEnd]
---*/

var trimEnd = String.prototype.trimEnd;

assert.sameValue(typeof trimEnd, 'function');

assert.throws(TypeError, function() {
  trimEnd.call(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  trimEnd.call(null);
}, 'null');
