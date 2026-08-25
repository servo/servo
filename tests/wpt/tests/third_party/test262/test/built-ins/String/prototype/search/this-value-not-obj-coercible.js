// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.search
description: The "this" value must be object-coercible
info: |
  1. Let O be ? RequireObjectCoercible(this value).
---*/

var search = String.prototype.search;

assert.sameValue(typeof search, 'function');

assert.throws(TypeError, function() {
  search.call(undefined, /./);
}, 'undefined');

assert.throws(TypeError, function() {
  search.call(null, /./);
}, 'null');
