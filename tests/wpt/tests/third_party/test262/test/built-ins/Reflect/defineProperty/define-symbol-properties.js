// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.3
description: >
  Define symbol properties.
info: |
  26.1.3 Reflect.defineProperty ( target, propertyKey, attributes )

  ...
  2. Let key be ToPropertyKey(propertyKey).
  ...

  7.1.14 ToPropertyKey ( argument )

  ...
  3. If Type(key) is Symbol, then
    a. Return key.
  ...
features: [Reflect, Symbol]
---*/

var o = {};
var desc;

var s1 = Symbol('1');

Reflect.defineProperty(o, s1, {
  value: 42,
  writable: true,
  enumerable: true
});

assert.sameValue(o[s1], 42);

desc = Object.getOwnPropertyDescriptor(o, s1);

assert.sameValue(desc.writable, true);
assert.sameValue(desc.configurable, false);
assert.sameValue(desc.enumerable, true);

var s2 = Symbol('2');

var f1 = function() {};
var f2 = function() {};
Reflect.defineProperty(o, s2, {
  get: f1,
  set: f2
});

desc = Object.getOwnPropertyDescriptor(o, s2);
assert.sameValue(desc.get, f1);
assert.sameValue(desc.set, f2);
