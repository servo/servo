// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.1
description: >
    Return target.[[GetPrototypeOf]]() if trap is undefined.
features: [Proxy]
---*/

var target = Object.create(Array.prototype);
var p = new Proxy(target, {});

assert.sameValue(Object.getPrototypeOf(p), Array.prototype);
