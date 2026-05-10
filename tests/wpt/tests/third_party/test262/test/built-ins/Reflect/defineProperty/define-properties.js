// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.3
description: >
  Define properties from the attributes object.
info: |
  26.1.3 Reflect.defineProperty ( target, propertyKey, attributes )

  ...
  6. Return target.[[DefineOwnProperty]](key, desc).
includes: [propertyHelper.js]
features: [Reflect]
---*/

var o = {};
var desc;

Reflect.defineProperty(o, 'p1', {
  value: 42,
  writable: true,
  enumerable: true
});

assert.sameValue(o.p1, 42);

verifyWritable(o, 'p1');
verifyNotConfigurable(o, 'p1');
verifyEnumerable(o, 'p1');

var f1 = function() {};
var f2 = function() {};
Reflect.defineProperty(o, 'p2', {
  get: f1,
  set: f2
});

desc = Object.getOwnPropertyDescriptor(o, 'p2');
assert.sameValue(desc.get, f1);
assert.sameValue(desc.set, f2);
