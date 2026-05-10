// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.1.2.5
description: >
    Frozen object contains symbol properties.
flags: [onlyStrict]
features: [Symbol]
---*/
var sym = Symbol("66");
var obj = {};
obj[sym] = 1;
Object.freeze(obj);

assert.throws(TypeError, function() {
  obj[sym] = 2;
});
