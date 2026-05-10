// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.9
description: >
  Return boolean value from a projectKey as a Symbol
info: |
  26.1.9 Reflect.has ( target, propertyKey )

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
var s1 = Symbol('1');
o[s1] = 42;

var s2 = Symbol('1');


assert.sameValue(Reflect.has(o, s1), true, 'true from own property');
assert.sameValue(
  Reflect.has(o, s2), false,
  'false when property is not present'
);
