// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.11
description: >
  Returns target's own non enumerable property keys.
info: |
  26.1.11 Reflect.ownKeys ( target )

  ...
  2. Let keys be target.[[OwnPropertyKeys]]().
  3. ReturnIfAbrupt(keys).
  4. Return CreateArrayFromList(keys).
includes: [compareArray.js]
features: [Reflect]
---*/

assert.compareArray(
  Reflect.ownKeys([]),
  ['length'],
  'return non enumerable `length` from empty array'
);

assert.compareArray(Reflect.ownKeys([, , 2]), ['2', 'length'], 'return array keys');

var o = {};
Object.defineProperty(o, 'p1', {
  value: 42,
  enumerable: false
});
Object.defineProperty(o, 'p2', {
  get: function() {},
  enumerable: false
});

assert.compareArray(Reflect.ownKeys(o), ['p1', 'p2']);
