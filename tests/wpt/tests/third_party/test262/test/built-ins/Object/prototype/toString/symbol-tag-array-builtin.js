// Copyright (C) 2019 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.prototype.tostring
description: >
  Non-string values of `Symbol.toStringTag` property are ignored.
info: |
  Object.prototype.toString ( )

  [...]
  15. Let tag be ? Get(O, @@toStringTag).
  16. If Type(tag) is not String, set tag to builtinTag.
  17. Return the string-concatenation of "[object ", tag, and "]".
features: [Symbol.toStringTag, Symbol.iterator, iterator-helpers]
---*/

var toString = Object.prototype.toString;

var arrIter = [][Symbol.iterator]();
var arrIterProto = Object.getPrototypeOf(arrIter);

assert.sameValue(toString.call(arrIter), '[object Array Iterator]');

Object.defineProperty(arrIterProto, Symbol.toStringTag, {configurable: true, value: null});
assert.sameValue(toString.call(arrIter), '[object Object]');

delete arrIterProto[Symbol.toStringTag];
assert.sameValue(toString.call(arrIter), '[object Iterator]');
