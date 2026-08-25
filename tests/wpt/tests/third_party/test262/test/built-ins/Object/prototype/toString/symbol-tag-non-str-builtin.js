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
features: [Symbol.toStringTag]
---*/

var toString = Object.prototype.toString;

delete Symbol.prototype[Symbol.toStringTag];
assert.sameValue(toString.call(Symbol('desc')), '[object Object]');

Object.defineProperty(Math, Symbol.toStringTag, {value: Symbol()});
assert.sameValue(toString.call(Math), '[object Object]');

delete JSON[Symbol.toStringTag];
assert.sameValue(toString.call(JSON), '[object Object]');

