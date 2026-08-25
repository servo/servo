// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.5
description: >
    Frozen object contains symbol properties.
flags: [noStrict]
features: [Symbol]
---*/
var sym = Symbol();
var obj = {};
obj[sym] = 1;
Object.freeze(obj);
obj[sym] = 2;
assert.sameValue(obj[sym], 1, "The value of `obj[sym]` is `1`");
assert.sameValue(delete obj[sym], false, "`delete obj[sym]` is `false`");
