// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-string.prototype.tolocaleuppercase
description: The "this" value must be object-coercible
info: |
  This function works exactly the same as toUpperCase except that its result is
  intended to yield the correct result for the host environment's current
  locale, rather than a locale-independent result.

  21.1.3.26 String.prototype.toUpperCase

  This function behaves in exactly the same way as
  String.prototype.toLowerCase, except that code points are mapped to their
  uppercase equivalents as specified in the Unicode Character Database.

  21.1.3.24 String.prototype.toLowerCase

  1. Let O be ? RequireObjectCoercible(this value).
---*/

var toLocaleUpperCase = String.prototype.toLocaleUpperCase;

assert.sameValue(typeof toLocaleUpperCase, 'function');

assert.throws(TypeError, function() {
  toLocaleUpperCase.call(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  toLocaleUpperCase.call(null);
}, 'null');
