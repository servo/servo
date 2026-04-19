// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.7
description: >
  Use a symbol value on property key.
info: |
  26.1.7 Reflect.getOwnPropertyDescriptor ( target, propertyKey )

  ...
  2. Let key be ToPropertyKey(propertyKey).
  ...

  7.1.14 ToPropertyKey ( argument )

  ...
  3. If Type(key) is Symbol, then
    a. Return key.
  ...
includes: [compareArray.js]
features: [Reflect, Symbol]
---*/

var o = {};
var s = Symbol('42');
o[s] = 42;

var result = Reflect.getOwnPropertyDescriptor(o, s);

assert.compareArray(
  Object.getOwnPropertyNames(result),
  ['value', 'writable', 'enumerable', 'configurable']
);
assert.sameValue(result.value, 42);
assert.sameValue(result.enumerable, true);
assert.sameValue(result.configurable, true);
assert.sameValue(result.writable, true);
