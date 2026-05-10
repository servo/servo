// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Generators declared with GeneratorDeclaration syntax do not require a
    `yield` expression.
es6id: 14.4
features: [generators]
---*/

function *foo(a) {}

var g = foo(3);

assert.sameValue(g.next().value, undefined);
assert.sameValue(g.next().done, true);
