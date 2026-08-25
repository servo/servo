// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 12.9.4
description: >
    Type coercion of value returned by constructor's @@hasInstance property
info: |
    1. If Type(C) is not Object, throw a TypeError exception.
    2. Let instOfHandler be GetMethod(C,@@hasInstance).
    3. ReturnIfAbrupt(instOfHandler).
    4. If instOfHandler is not undefined, then
       a. Return ToBoolean(Call(instOfHandler, C, «O»)).
features: [Symbol, Symbol.hasInstance]
---*/

var F = {};

F[Symbol.hasInstance] = function() { return undefined; };
assert.sameValue(0 instanceof F, false);

F[Symbol.hasInstance] = function() { return null; };
assert.sameValue(0 instanceof F, false);

F[Symbol.hasInstance] = function() { return true; };
assert.sameValue(0 instanceof F, true);

F[Symbol.hasInstance] = function() { return NaN; };
assert.sameValue(0 instanceof F, false);

F[Symbol.hasInstance] = function() { return 1; };
assert.sameValue(0 instanceof F, true);

F[Symbol.hasInstance] = function() { return ''; };
assert.sameValue(0 instanceof F, false);

F[Symbol.hasInstance] = function() { return 'string'; };
assert.sameValue(0 instanceof F, true);

F[Symbol.hasInstance] = function() { return Symbol(); };
assert.sameValue(0 instanceof F, true);

F[Symbol.hasInstance] = function() { return {}; };
assert.sameValue(0 instanceof F, true);
