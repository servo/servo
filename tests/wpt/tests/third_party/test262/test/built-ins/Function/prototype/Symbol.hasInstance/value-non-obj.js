// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.2.3.6
description: Non-object argument
info: |
    1. Let F be the this value.
    2. Return OrdinaryHasInstance(F, V).

    7.3.19 OrdinaryHasInstance (C, O)

    [...]
    3. If Type(O) is not Object, return false.
features: [Symbol, Symbol.hasInstance]
---*/

assert.sameValue(function() {}[Symbol.hasInstance](), false);
assert.sameValue(function() {}[Symbol.hasInstance](undefined), false);
assert.sameValue(function() {}[Symbol.hasInstance](null), false);
assert.sameValue(function() {}[Symbol.hasInstance](true), false);
assert.sameValue(function() {}[Symbol.hasInstance]('string'), false);
assert.sameValue(function() {}[Symbol.hasInstance](Symbol()), false);
assert.sameValue(function() {}[Symbol.hasInstance](86), false);
