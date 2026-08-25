// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.trimstart
description: The "this" value must be object-coercible
info: |
  1. Let O be ? RequireObjectCoercible(this value).
features: [string-trimming, String.prototype.trimStart]
---*/

var trimStart = String.prototype.trimStart;

assert.sameValue(typeof trimStart, 'function');

assert.throws(TypeError, function() {
  trimStart.call(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  trimStart.call(null);
}, 'null');
