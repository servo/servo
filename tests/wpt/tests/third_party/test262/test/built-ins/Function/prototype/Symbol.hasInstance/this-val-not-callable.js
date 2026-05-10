// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.2.3.6
description: Non-callable `this` value
info: |
    1. Let F be the this value.
    2. Return OrdinaryHasInstance(F, V).

    7.3.19 OrdinaryHasInstance (C, O)

    1. If IsCallable(C) is false, return false.
features: [Symbol.hasInstance]
---*/

assert.sameValue(Function.prototype[Symbol.hasInstance].call(), false);
assert.sameValue(Function.prototype[Symbol.hasInstance].call({}), false);
