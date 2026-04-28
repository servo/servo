// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.create
description: >
  The Properties argument is cast to an object if not undefined
info: |
  Object.create ( O, Properties )

  3. If Properties is not undefined, then
    a. Return ? ObjectDefineProperties(obj, Properties).

  Runtime Semantics: ObjectDefineProperties ( O, Properties )

  2. Let props be ? ToObject(Properties).
  3. Let keys be ? props.[[OwnPropertyKeys]]().
  ...
  // All enumerable keys are added to the created object.
features: [Symbol]
---*/

var proto = {};

var obj;
obj = Object.create(proto, true);
assert.sameValue(Object.getPrototypeOf(obj), proto, 'Properties is true: prototype is set');
assert.sameValue(Object.getOwnPropertyNames(obj).length, 0, 'Properties is true: no keys set');
assert.sameValue(Object.getOwnPropertySymbols(obj).length, 0, 'Properties is true: no symbol keys set');

obj = undefined;
obj = Object.create(proto, false);
assert.sameValue(Object.getPrototypeOf(obj), proto, 'Properties is false: prototype is set');
assert.sameValue(Object.getOwnPropertyNames(obj).length, 0, 'Properties is false: no keys set');
assert.sameValue(Object.getOwnPropertySymbols(obj).length, 0, 'Properties is false: no symbol keys set');

obj = undefined;
obj = Object.create(proto, 1);
assert.sameValue(Object.getPrototypeOf(obj), proto, 'Properties is 1: prototype is set');
assert.sameValue(Object.getOwnPropertyNames(obj).length, 0, 'Properties is 1: no keys set');
assert.sameValue(Object.getOwnPropertySymbols(obj).length, 0, 'Properties is 1: no symbol keys set');

obj = undefined;
obj = Object.create(proto, NaN);
assert.sameValue(Object.getPrototypeOf(obj), proto, 'Properties is NaN: prototype is set');
assert.sameValue(Object.getOwnPropertyNames(obj).length, 0, 'Properties is NaN: no keys set');
assert.sameValue(Object.getOwnPropertySymbols(obj).length, 0, 'Properties is NaN: no symbol keys set');

obj = undefined;
obj = Object.create(proto, '');
assert.sameValue(Object.getPrototypeOf(obj), proto, 'Properties is the empty string: prototype is set');
assert.sameValue(Object.getOwnPropertyNames(obj).length, 0, 'Properties is the empty string: no keys set');
assert.sameValue(Object.getOwnPropertySymbols(obj).length, 0, 'Properties is the empty string: no symbol keys set');

obj = undefined;
obj = Object.create(proto, Symbol('s'));
assert.sameValue(Object.getPrototypeOf(obj), proto, 'Properties is symbol: prototype is set');
assert.sameValue(Object.getOwnPropertyNames(obj).length, 0, 'Properties is symbol: no keys set');
assert.sameValue(Object.getOwnPropertySymbols(obj).length, 0, 'Properties is symbol: no symbol keys set');
