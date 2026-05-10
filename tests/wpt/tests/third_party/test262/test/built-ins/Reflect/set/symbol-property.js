// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.13
description: >
  Sets the new value.
info: |
  26.1.13 Reflect.set ( target, propertyKey, V [ , receiver ] )

  ...
  2. Let key be ToPropertyKey(propertyKey).
  ...

  7.1.14 ToPropertyKey ( argument )

  ...
  3. If Type(key) is Symbol, then
    a. Return key.
  ...
features: [Reflect, Reflect.set, Symbol]
---*/

var o1 = {};
var s = Symbol('1');
var result = Reflect.set(o1, s, 42);
assert.sameValue(result, true, 'returns true on a successful setting');
assert.sameValue(o1[s], 42, 'sets the new value');

var o2 = {};
o2[s] = 43;
var receiver = {};
receiver[s] = 44;
var result = Reflect.set(o2, s, 42, receiver);
assert.sameValue(result, true, 'returns true on a successful setting');
assert.sameValue(o2[s], 43, 'with a receiver, does not set a value on target');
assert.sameValue(receiver[s], 42, 'sets the new value on the receiver');
