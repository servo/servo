// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.12.3
description: >
    "Logical AND" Symbol evaluation
features: [Symbol]
---*/
var sym = Symbol();

assert.sameValue(sym && true, true, "`sym && true` is `true`");
assert.sameValue(!sym && false, false, "`!sym && false` is `false`");
