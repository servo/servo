// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    The value of the [[Prototype]] internal slot of the GeneratorFunction
    prototype object is the FunctionPrototype intrinsic object.
es6id: 25.2.2.2
features: [generators]
---*/

function f() {}
var g = function*() {};

assert.sameValue(
  Object.getPrototypeOf(Object.getPrototypeOf(g)),
  Object.getPrototypeOf(f)
);
