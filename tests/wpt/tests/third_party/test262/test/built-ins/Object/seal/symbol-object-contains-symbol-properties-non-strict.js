// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.seal
description: >
    Object.seal(obj) where obj contains symbol properties.
flags: [noStrict]
features: [Symbol]
---*/
var symA = Symbol("A");
var symB = Symbol("B");
var obj = {};
obj[symA] = 1;
Object.seal(obj);
obj[symA] = 2;
obj[symB] = 1;

assert.sameValue(obj[symA], 2, "The value of `obj[symA]` is `2`");
assert.sameValue(delete obj[symA], false, "`delete obj[symA]` is `false`");
assert.sameValue(obj[symB], undefined, "The value of `obj[symB]` is `undefined`");
