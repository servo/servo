// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.4
description: >
    Symbol used as property for writable data property definition
features: [Symbol]
includes: [propertyHelper.js]
---*/
var sym = Symbol();
var obj = {};


Object.defineProperty(obj, sym, {
  value: 1,
  writable: true
});

assert.sameValue(sym in obj, true, "The result of `sym in obj` is `true`");
verifyProperty(obj, sym, {
  value: 1,
  configurable: false,
  writable: true,
  enumerable: false,
});

assert.sameValue(
  Object.prototype.propertyIsEnumerable.call(obj, sym),
  false,
  "`Object.prototype.propertyIsEnumerable.call(obj, sym)` returns `false`"
);

obj[sym] = 2;

assert.sameValue(obj[sym], 2, "The value of `obj[sym]` is `2`");
