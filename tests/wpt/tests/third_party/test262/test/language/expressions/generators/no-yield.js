// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Generators declared with GeneratorExpression syntax do not require a
    `yield` expression.
es6id: 14.4
features: [generators]
---*/

var result;
var foo = function*(a) {};

result = foo(3).next();

assert.sameValue(result.value, undefined);
assert.sameValue(result.done, true);
