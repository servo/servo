// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-object.is
description: >
    Object.is/SameValue: Symbol
features: [Object.is, Symbol]
---*/
var symA = Symbol('66');
var symB = Symbol('66');


assert.sameValue(Object.is(symA, symA), true, "`Object.is(symA, symA)` returns `true`");
assert.sameValue(Object.is(symB, symB), true, "`Object.is(symB, symB)` returns `true`");
assert.sameValue(Object.is(symA, symB), false, "`Object.is(symA, symB)` returns `false`");
