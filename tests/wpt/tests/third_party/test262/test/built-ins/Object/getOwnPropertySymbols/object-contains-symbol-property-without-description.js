// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.getownpropertysymbols
description: >
    Object.getOwnPropertySymbols returns all symbol properties that do not have descriptions
features: [Symbol]
---*/

var sym = Symbol();

var obj = {};
obj[sym] = 1;

var syms = Object.getOwnPropertySymbols(obj);

assert.sameValue(syms[0], sym, "Array of symbols returned by `Object.getOwnPropertySymbols(obj)` includes `sym`");
assert.sameValue(syms.length, 1, "The value of `syms.length` is `1`");
