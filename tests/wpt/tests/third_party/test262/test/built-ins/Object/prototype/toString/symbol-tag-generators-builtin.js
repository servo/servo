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
features: [Symbol.toStringTag, Symbol.iterator, generators]
---*/

var toString = Object.prototype.toString;

var genFn = function* () {};
assert.sameValue(toString.call(genFn), '[object GeneratorFunction]');

var gen = genFn();
assert.sameValue(toString.call(gen), '[object Generator]');

var genProto = Object.getPrototypeOf(gen);
assert.sameValue(genProto, genFn.prototype);

Object.defineProperty(genProto, Symbol.toStringTag, {
  configurable: true,
  get: function() { return {}; },
});
assert.sameValue(toString.call(gen), '[object Object]');

delete genProto[Symbol.toStringTag];
assert.sameValue(toString.call(gen), '[object Generator]');
