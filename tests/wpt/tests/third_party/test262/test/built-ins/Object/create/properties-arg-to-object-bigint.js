// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.create
description: >
  The Properties argument is cast to an object if it's a BigInt value
info: |
  Object.create ( O, Properties )

  3. If Properties is not undefined, then
    a. Return ? ObjectDefineProperties(obj, Properties).

  Runtime Semantics: ObjectDefineProperties ( O, Properties )

  2. Let props be ? ToObject(Properties).
  3. Let keys be ? props.[[OwnPropertyKeys]]().
  ...
  // All enumerable keys are added to the created object.
features: [BigInt]
---*/

var proto = {};

var obj;
obj = Object.create(proto, 1n);
assert.sameValue(Object.getPrototypeOf(obj), proto, 'Properties is 1n: prototype is set');
assert.sameValue(Object.getOwnPropertyNames(obj).length, 0, 'Properties is 1n: no keys set');
assert.sameValue(Object.getOwnPropertySymbols(obj).length, 0, 'Properties is 1n: no symbol keys set');

obj = undefined;
obj = Object.create(proto, 0n);
assert.sameValue(Object.getPrototypeOf(obj), proto, 'Properties is 0n: prototype is set');
assert.sameValue(Object.getOwnPropertyNames(obj).length, 0, 'Properties is 0n: no keys set');
assert.sameValue(Object.getOwnPropertySymbols(obj).length, 0, 'Properties is 0n: no symbol keys set');
