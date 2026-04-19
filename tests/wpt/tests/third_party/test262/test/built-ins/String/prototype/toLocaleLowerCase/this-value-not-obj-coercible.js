// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.tolocalelowercase
description: The "this" value must be object-coercible
info: |
  This function works exactly the same as toLowerCase except that its result is
  intended to yield the correct result for the host environment's current
  locale, rather than a locale-independent result.

  21.1.3.24 String.prototype.toLowerCase

  1. Let O be ? RequireObjectCoercible(this value).
---*/

var toLocaleLowerCase = String.prototype.toLocaleLowerCase;

assert.sameValue(typeof toLocaleLowerCase, 'function');

assert.throws(TypeError, function() {
  toLocaleLowerCase.call(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  toLocaleLowerCase.call(null);
}, 'null');
