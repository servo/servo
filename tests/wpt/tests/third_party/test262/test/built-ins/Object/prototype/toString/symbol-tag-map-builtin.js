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
features: [Symbol.toStringTag, Symbol.iterator, Map, iterator-helpers]
---*/

var toString = Object.prototype.toString;

var map = new Map();
delete Map.prototype[Symbol.toStringTag];
assert.sameValue(toString.call(map), '[object Object]');

var mapIter = map[Symbol.iterator]();
var mapIterProto = Object.getPrototypeOf(mapIter);
assert.sameValue(toString.call(mapIter), '[object Map Iterator]');
Object.defineProperty(mapIterProto, Symbol.toStringTag, {
  configurable: true,
  get: function() { return new String('ShouldNotBeUnwrapped'); },
});
assert.sameValue(toString.call(mapIter), '[object Object]');

delete mapIterProto[Symbol.toStringTag];
assert.sameValue(toString.call(mapIter), '[object Iterator]');
