// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.2
description: >
  Return target result using newTarget argument.
info: |
  26.1.2 Reflect.construct ( target, argumentsList [, newTarget] )

  ...
  2. If newTarget is not present, let newTarget be target.
  ...
  6. Return Construct(target, args, newTarget).
features: [Reflect, Reflect.construct]
---*/

var o = {};
var internPrototype;

function fn() {
  this.o = o;
  internPrototype = Object.getPrototypeOf(this);
}

var result = Reflect.construct(fn, [], Array);
assert.sameValue(Object.getPrototypeOf(result), Array.prototype);
assert.sameValue(
  internPrototype, Array.prototype,
  'prototype of this from within the constructor function is Array.prototype'
);
assert.sameValue(result.o, o);
