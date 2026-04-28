// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.3
description: >
  Return boolean result of the property definition.
info: |
  26.1.3 Reflect.defineProperty ( target, propertyKey, attributes )

  ...
  6. Return target.[[DefineOwnProperty]](key, desc).
features: [Reflect]
---*/

var o = {};

o.p1 = 'foo';
assert.sameValue(Reflect.defineProperty(o, 'p1', {}), true);
assert.sameValue(o.hasOwnProperty('p1'), true);

assert.sameValue(Reflect.defineProperty(o, 'p2', {
  value: 42
}), true);
assert.sameValue(o.hasOwnProperty('p2'), true);

Object.freeze(o);

assert.sameValue(Reflect.defineProperty(o, 'p2', {
  value: 43
}), false);
assert.sameValue(o.p2, 42);

assert.sameValue(Reflect.defineProperty(o, 'p3', {}), false);
assert.sameValue(o.hasOwnProperty('p4'), false);

assert.sameValue(Reflect.defineProperty(o, 'p4', {
  value: 1
}), false);
assert.sameValue(o.hasOwnProperty('p4'), false);
