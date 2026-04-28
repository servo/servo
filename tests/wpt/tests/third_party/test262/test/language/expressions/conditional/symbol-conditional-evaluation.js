// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.12.3
description: >
    Conditional Symbol evaluation
features: [Symbol]
---*/
var sym = Symbol();

assert.sameValue(sym ? 1 : 2, 1, "`sym ? 1 : 2` is `1`");
assert.sameValue(!sym ? 1 : 2, 2, "`!sym ? 1 : 2` is `2`");
